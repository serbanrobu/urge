use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use strum::{Display, EnumDiscriminants, EnumIter, EnumString, IntoEnumIterator};

#[cfg(feature = "ssr")]
pub fn register_server_functions() {
    // TODO
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos_start.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                    <Route path="playground" view=|cx| view! { cx, <PlaygroundPage/> }/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
            <div class="mx-auto max-w-3xl">
                <h1>"Home"</h1>
                <a href="/playground">"Playground"</a>
            </div>
        </div>
    }
}

type Name = String;

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, EnumString, Display))]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    F64Lit(f64),
    Var(Name),
}

enum ExprPatch {
    Add(Box<AddPatch>),
    Expr(Expr),
    F64Lit(f64),
    Var(Name),
}

impl IntoView for ExprDiscriminants {
    fn into_view(self, cx: Scope) -> View {
        self.to_string().into_view(cx)
    }
}

#[component]
fn Expr<F>(cx: Scope, value: Expr, on_change: F) -> impl IntoView
where
    F: Fn(ExprPatch) + Copy + 'static,
{
    let initial_discriminant = ExprDiscriminants::from(&value);
    let expr = create_rw_signal(cx, value);

    view! {
        cx,
        <select
            on:change=move |ev| {
                let value = event_target_value(&ev);

                let Ok(value) = value.parse::<ExprDiscriminants>() else {
                    return;
                };

                match value {
                    ExprDiscriminants::Add => {
                        let value = Expr::Add(
                            Box::new(Expr::F64Lit(0.0)),
                            Box::new(Expr::F64Lit(0.0)),
                        );

                        expr.set(value.clone());
                        on_change(ExprPatch::Expr(value));
                    }
                    ExprDiscriminants::F64Lit => expr.set(Expr::F64Lit(0.0)),
                    ExprDiscriminants::Var => expr.set(Expr::Var(String::new())),
                }
            }
        >
            {ExprDiscriminants::iter()
                .map(|d| view! {
                    cx,
                    <option selected=initial_discriminant == d>{d}</option>
                })
                .collect::<Vec<_>>()}
        </select>

        {move || match expr() {
            Expr::Add(e_1, e_2) => {
                view! {
                    cx,
                    // <Expr value=(*e_1).clone() on_change=|_| ()/>
                    " + "
                    // <Expr value=(*e_2).clone() on_change=|_| ()/>

                    // <Add
                    //     lhs=(*e_1).clone()
                    //     rhs=(*e_2).clone()
                    //     on_change=move |patch| on_change(ExprPatch::Add(Box::new(patch)))
                    // />
                }
                .into_view(cx)
            }
            Expr::F64Lit(n) => {
                view! {
                    cx,
                    <F64 value=n on_change=move |value| on_change(ExprPatch::F64Lit(value))/>
                }
                .into_view(cx)
            }
            Expr::Var(x) => {
                view! {
                    cx,
                    <Name value=x on_change=move |value| on_change(ExprPatch::Var(value))/>
                }
                .into_view(cx)
            }
        }}
    }
}

enum AddPatch {
    Lhs(ExprPatch),
    Rhs(ExprPatch),
}

#[component]
fn Add<F>(cx: Scope, lhs: Expr, rhs: Expr, on_change: F) -> impl IntoView
where
    F: Fn(AddPatch) + Copy + 'static,
{
    view! {
        cx,
        <Expr value=lhs on_change=move |patch| on_change(AddPatch::Lhs(patch))/>
        " + "
        <Expr value=rhs on_change=move |patch| on_change(AddPatch::Rhs(patch))/>
    }
}

#[component]
fn F64<F>(cx: Scope, value: f64, on_change: F) -> impl IntoView
where
    F: Fn(f64) + 'static,
{
    view! {
        cx,
        <input
            type="number"
            value=value
            on:change=move |ev| {
                let value = event_target_value(&ev);

                let Ok(value) = value.parse::<f64>() else {
                    return;
                };

                on_change(value);
            }
            placeholder="f64"
        />
    }
}

#[component]
fn Name<F>(cx: Scope, value: Name, on_change: F) -> impl IntoView
where
    F: Fn(Name) + 'static,
{
    view! {
        cx,
        <input
            type="text"
            value=value
            on:change=move |ev| on_change(event_target_value(&ev))
            placeholder="Name"
        />
    }
}

#[component]
fn PlaygroundPage(cx: Scope) -> impl IntoView {
    let expr = Expr::F64Lit(0.0);

    view! {
        cx,
        <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
            <div class="mx-auto max-w-3xl">
                <Expr value=expr on_change=|_| ()/>
            </div>
        </div>
    }
}
