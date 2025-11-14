export function render_chart(labels, values) {
  const data = [labels, values];
  const opts = {
    width: 800,
    height: 400,
    series: [
      {},
      {
        label: "Price",
        stroke: "rgb(0, 150, 255)",
      },
    ],
    scales: {
      x: { time: true },
    },
  };

  const chartEl = document.getElementById("chart");
  chartEl.innerHTML = ""; // clear if re-rendered
  new uPlot(opts, data, chartEl);
}
