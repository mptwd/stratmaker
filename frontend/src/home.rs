use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

// Landing Page Component
#[function_component(LandingPage)]
pub fn landing_page() -> Html {
    html! {
        <div class="landing-page">
            <nav class="navbar">
                <div class="container">
                    <h1 class="logo">{"StrategyMaker"}</h1>
                    <div class="nav-links">
                        <Link<Route> to={Route::Login} classes="btn-secondary">{"Login"}</Link<Route>>
                        <Link<Route> to={Route::Register} classes="btn-primary">{"Get Started"}</Link<Route>>
                    </div>
                </div>
            </nav>

            <section class="hero">
                <div class="container">
                    <h1 class="hero-title">{"Build Your Financial Future"}</h1>
                    <p class="hero-subtitle">
                        {"Create, test, and optimize your investment strategies with our powerful platform"}
                    </p>
                    <div class="hero-cta">
                        <Link<Route> to={Route::Register} classes="btn-primary btn-large">
                            {"Start Building Free"}
                        </Link<Route>>
                    </div>
                </div>
            </section>

            <section class="features">
                <div class="container">
                    <h2 class="section-title">{"Why Choose StrategyMaker?"}</h2>
                    <div class="features-grid">
                        <div class="feature-card">
                            <div class="feature-icon">{"📊"}</div>
                            <h3>{"Advanced Analytics"}</h3>
                            <p>{"Real-time analysis of your financial strategies with comprehensive metrics"}</p>
                        </div>
                        <div class="feature-card">
                            <div class="feature-icon">{"🔄"}</div>
                            <h3>{"Backtesting"}</h3>
                            <p>{"Test your strategies against historical data to validate performance"}</p>
                        </div>
                        <div class="feature-card">
                            <div class="feature-icon">{"🎯"}</div>
                            <h3>{"Risk Management"}</h3>
                            <p>{"Optimize your portfolio with advanced risk assessment tools"}</p>
                        </div>
                        <div class="feature-card">
                            <div class="feature-icon">{"⚡"}</div>
                            <h3>{"Lightning Fast"}</h3>
                            <p>{"Built with Rust for maximum performance and reliability"}</p>
                        </div>
                    </div>
                </div>
            </section>

            <section class="pricing">
                <div class="container">
                    <h2 class="section-title">{"Simple, Transparent Pricing"}</h2>
                    <div class="pricing-grid">
                        <div class="pricing-card">
                            <h3>{"Free"}</h3>
                            <div class="price">{"$0"}<span class="period">{"/month"}</span></div>
                            <ul class="features-list">
                                <li>{"✓ Up to 3 strategies"}</li>
                                <li>{"✓ Basic analytics"}</li>
                                <li>{"✓ 1 year historical data"}</li>
                                <li>{"✓ Community support"}</li>
                            </ul>
                            <Link<Route> to={Route::Register} classes="btn-outline">{"Get Started"}</Link<Route>>
                        </div>
                        <div class="pricing-card featured">
                            <div class="badge">{"Popular"}</div>
                            <h3>{"Pro"}</h3>
                            <div class="price">{"$29"}<span class="period">{"/month"}</span></div>
                            <ul class="features-list">
                                <li>{"✓ Unlimited strategies"}</li>
                                <li>{"✓ Advanced analytics"}</li>
                                <li>{"✓ 10 years historical data"}</li>
                                <li>{"✓ Priority support"}</li>
                                <li>{"✓ API access"}</li>
                            </ul>
                            <Link<Route> to={Route::Register} classes="btn-primary">{"Start Free Trial"}</Link<Route>>
                        </div>
                        <div class="pricing-card">
                            <h3>{"Enterprise"}</h3>
                            <div class="price">{"Custom"}</div>
                            <ul class="features-list">
                                <li>{"✓ Everything in Pro"}</li>
                                <li>{"✓ Custom integrations"}</li>
                                <li>{"✓ Dedicated support"}</li>
                                <li>{"✓ On-premise option"}</li>
                                <li>{"✓ SLA guarantee"}</li>
                            </ul>
                            <button class="btn-outline">{"Contact Sales"}</button>
                        </div>
                    </div>
                </div>
            </section>

            <footer class="footer">
                <div class="container">
                    <p>{"© 2025 StrategyMaker. Built with Rust & Yew."}</p>
                </div>
            </footer>
        </div>
    }
}
