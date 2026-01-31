import requests
import pandas as pd
import struct
import gzip
import json
import hashlib
import time
import boto3
import typer
import os
from pathlib import Path
from datetime import datetime
from dateutil import parser as dateparser
from typing import Optional, List
from concurrent.futures import ThreadPoolExecutor, as_completed
from tqdm import tqdm

app = typer.Typer(add_completion=False)

BINANCE_URL = "https://api.binance.com/api/v3/klines"
MAX_LIMIT = 1000


# =========================
# Utilities
# =========================

def parse_date(s: Optional[str]) -> Optional[int]:
    if s is None:
        return None
    dt = dateparser.parse(s)
    return int(dt.timestamp() * 1000)


def interval_to_ms(interval: str) -> int:
    unit = interval[-1]
    n = int(interval[:-1])
    if unit == "m":
        return n * 60_000
    if unit == "h":
        return n * 60 * 60_000
    if unit == "d":
        return n * 24 * 60 * 60_000
    raise ValueError(f"Unsupported interval: {interval}")


def make_r2_client(account_id: str, access_key: str, secret_key: str):
    return boto3.client(
        "s3",
        endpoint_url=f"https://{account_id}.r2.cloudflarestorage.com",
        aws_access_key_id=access_key,
        aws_secret_access_key=secret_key,
        region_name="auto",
    )


# =========================
# Binance API
# =========================

def fetch_klines_chunk(symbol: str, interval: str, start_ms: int, end_ms: int) -> pd.DataFrame:
    rows = []
    start = start_ms

    while True:
        params = {
            "symbol": symbol,
            "interval": interval,
            "startTime": start,
            "limit": MAX_LIMIT,
        }
        resp = requests.get(BINANCE_URL, params=params, timeout=20)
        resp.raise_for_status()
        data = resp.json()

        if not data:
            break

        for k in data:
            ts = k[0]
            if ts >= end_ms:
                return pd.DataFrame(rows)
            rows.append({
                "timestamp": ts // 1000,
                "open": float(k[1]),
                "high": float(k[2]),
                "low": float(k[3]),
                "close": float(k[4]),
                "volume": float(k[5]),
            })

        start = data[-1][0] + 1
        time.sleep(0.12)

    return pd.DataFrame(rows)


# =========================
# Parallel orchestration
# =========================

def split_ranges(start_ms: int, end_ms: int, chunk_ms: int) -> List[tuple]:
    ranges = []
    cur = start_ms
    while cur < end_ms:
        ranges.append((cur, min(cur + chunk_ms, end_ms)))
        cur += chunk_ms
    return ranges


def download_parallel(symbol: str, interval: str, start_ms: int, end_ms: int, workers: int) -> pd.DataFrame:
    candle_ms = interval_to_ms(interval)
    chunk_ms = MAX_LIMIT * candle_ms

    ranges = split_ranges(start_ms, end_ms, chunk_ms)

    print(f"⚡ Downloading {len(ranges)} chunks using {workers} workers...")

    dfs = []

    with ThreadPoolExecutor(max_workers=workers) as ex:
        futures = [
            ex.submit(fetch_klines_chunk, symbol, interval, s, e)
            for s, e in ranges
        ]
        for f in tqdm(as_completed(futures), total=len(futures)):
            dfs.append(f.result())

    if not dfs:
        return pd.DataFrame()

    df = pd.concat(dfs, ignore_index=True)
    df.drop_duplicates(subset="timestamp", inplace=True)
    df.sort_values("timestamp", inplace=True)
    df.reset_index(drop=True, inplace=True)
    return df


# =========================
# Binary encoding (f32)
# =========================

RECORD_STRUCT = struct.Struct("<Qfffff")  # timestamp + 5 floats = 28 bytes


def write_binary(df: pd.DataFrame, out_path: Path):
    hasher = hashlib.sha256()
    count = 0

    with gzip.open(out_path, "wb", compresslevel=6) as f:
        for row in df.itertuples(index=False):
            record = RECORD_STRUCT.pack(
                int(row.timestamp),
                float(row.open),
                float(row.high),
                float(row.low),
                float(row.close),
                float(row.volume),
            )
            f.write(record)
            hasher.update(record)
            count += 1

    return count, hasher.hexdigest()


def write_meta(df: pd.DataFrame, count: int, sha256: str,
               symbol: str, interval: str, out_path: Path):
    meta = {
        "symbol": symbol,
        "timeframe": interval,
        "candles": count,
        "start_ts": int(df["timestamp"].iloc[0]),
        "end_ts": int(df["timestamp"].iloc[-1]),
        "hash": f"sha256:{sha256}",
        "encoding": "u64,f32,f32,f32,f32,f32",
        "bytes_per_record": RECORD_STRUCT.size,
        "endianness": "little",
        "fields": ["timestamp", "open", "high", "low", "close", "volume"],
    }

    with open(out_path, "w") as f:
        json.dump(meta, f, indent=2)


# =========================
# Upload
# =========================

def upload_file(client, bucket: str, key: str, path: Path, content_type: str):
    client.upload_file(
        str(path),
        bucket,
        key,
        ExtraArgs={"ContentType": content_type},
    )


# =========================
# CLI
# =========================

@app.command()
def ingest(
    symbol: str = typer.Option(..., help="Trading pair, e.g. BTCUSDT"),
    interval: str = typer.Option("1m", help="Binance interval, e.g. 1m, 5m, 1h"),
    bucket: str = typer.Option(..., help="Cloudflare R2 bucket name"),
    prefix: str = typer.Option("crypto", help="Dataset root prefix in bucket"),
    out_dir: Path = typer.Option(Path("./tmp_dataset"), help="Local temp directory"),
    start_date: Optional[str] = typer.Option(None, help="Start date (YYYY-MM-DD)"),
    end_date: Optional[str] = typer.Option(None, help="End date (YYYY-MM-DD)"),
    workers: int = typer.Option(6, help="Parallel download workers"),
    account_id: Optional[str] = typer.Option(None, help="Cloudflare account ID"),
    access_key: Optional[str] = typer.Option(None, help="R2 access key"),
    secret_key: Optional[str] = typer.Option(None, help="R2 secret key"),
):
    """
    Download Binance candles in parallel, encode as compact f32 binary, and upload to Cloudflare R2.
    """

    account_id = account_id or os.getenv("R2_ACCOUNT_ID")
    access_key = access_key or os.getenv("R2_ACCESS_KEY")
    secret_key = secret_key or os.getenv("R2_SECRET_KEY")

    if not account_id or not access_key or not secret_key:
        raise typer.BadParameter("Missing R2 credentials (flags or env vars)")

    start_ms = parse_date(start_date) or 0
    end_ms = parse_date(end_date) or int(time.time() * 1000)

    out_dir.mkdir(parents=True, exist_ok=True)
    bin_path = out_dir / "candles.bin.gz"
    meta_path = out_dir / "meta.json"

    # 1️⃣ Parallel download
    df = download_parallel(symbol, interval, start_ms, end_ms, workers)

    if df.empty:
        raise RuntimeError("No data downloaded")

    # 2️⃣ Write binary
    print("📦 Writing binary (f32)...")
    count, sha256 = write_binary(df, bin_path)

    # 3️⃣ Write metadata
    print("🧾 Writing metadata...")
    write_meta(df, count, sha256, symbol, interval, meta_path)

    # 4️⃣ Upload
    print("☁️ Uploading to Cloudflare R2...")
    client = make_r2_client(account_id, access_key, secret_key)

    r2_prefix = f"{prefix}/{symbol}/{interval}/"
    upload_file(client, bucket, r2_prefix + "candles.bin.gz", bin_path, "application/gzip")
    upload_file(client, bucket, r2_prefix + "meta.json", meta_path, "application/json")

    print("✅ Done!")
    print(f"   Candles: {count:,}")
    print(f"   Record size: {RECORD_STRUCT.size} bytes")
    print(f"   R2 path: s3://{bucket}/{r2_prefix}")


if __name__ == "__main__":
    app()

