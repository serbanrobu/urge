type Level = u8;

type Name = String;

type Type = Value;

type Context = HashMap<Name, Type>;

type Env = HashMap<Name, Value>;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Add,
    Atom,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ExprKind {
    Add,
    Command,
    F64,
    F64Lit,
    Let,
    Trivial,
    Sole,
    U,
    Var,
}

impl FromStr for ExprKind {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Add" => Ok(Self::Add),
            "Command" => Ok(Self::Command),
            "F64" => Ok(Self::F64),
            "F64Lit" => Ok(Self::F64Lit),
            "Let" => Ok(Self::Let),
            "Trivial" => Ok(Self::Trivial),
            "Sole" => Ok(Self::Sole),
            "U" => Ok(Self::U),
            "Var" => Ok(Self::Var),
            _ => Err(eyre!("Not a valid ExprKind: {s:?}")),
        }
    }
}

impl IntoView for ExprKind {
    fn into_view(self, cx: Scope) -> View {
        self.to_string().into_view(cx)
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add => "Add".fmt(f),
            Self::Command => "Command".fmt(f),
            Self::F64 => "F64".fmt(f),
            Self::F64Lit => "F64Lit".fmt(f),
            Self::Let => "Let".fmt(f),
            Self::Trivial => "Trivial".fmt(f),
            Self::Sole => "Sole".fmt(f),
            Self::U => "U".fmt(f),
            Self::Var => "Var".fmt(f),
        }
    }
}

#[derive(PartialEq)]
pub enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Command(Box<Expr>),
    F64,
    F64Lit(f64),
    Let(Name, Box<Expr>, Box<Expr>),
    Trivial,
    Sole,
    U(Level),
    Var(Name),
}

impl Expr {
    pub fn check(&self, t: &Type, cx: &Context, env: &Env) -> Result<()> {
        match (self, t) {
            (Self::Command(e), Type::U(_)) => e.check(t, cx, env),
            (Self::F64, Type::U(_)) => Ok(()),
            (Self::F64Lit(_), Type::F64) => Ok(()),
            (Self::Let(_, e_1, e_2), Type::Command(t)) => {
                let Type::Trivial = **t else {
                    return Err(eyre!(""));
                };

                e_1.check(&Type::U(Level::MAX), cx, env)?;
                let t_1 = e_1.eval(env)?;
                e_2.check(&t_1, cx, env)
            }
            (Self::Trivial, Type::U(_)) => Ok(()),
            (Self::Sole, Type::Trivial) => Ok(()),
            (Self::U(i), Type::U(j)) if i < j => Ok(()),
            _ => {
                let e_1 = self.infer(cx, env)?.quote();
                let e_2 = t.quote();

                if e_1 != e_2 {
                    return Err(eyre!("Type mismatch: {e_1} vs {e_2}"));
                }

                Ok(())
            }
        }
    }

    fn eval(&self, env: &Env) -> Result<Value> {
        match self {
            Self::Add(e_1, e_2) => {
                let v_1 = e_1.eval(env)?;
                let v_2 = e_2.eval(env)?;

                match (v_1, v_2) {
                    (Value::F64Lit(lhs), Value::F64Lit(rhs)) => Ok(Value::F64Lit(lhs + rhs)),
                    (v_1, v_2) => Err(eyre!("Cannot add {} to {}", v_1.quote(), v_2.quote())),
                }
            }
            _ => todo!(),
        }
    }

    fn infer(&self, cx: &Context, env: &Env) -> Result<Type> {
        match self {
            Self::Add(e_1, e_2) => {
                let t = Type::F64;
                e_1.check(&t, cx, env)?;
                e_2.check(&t, cx, env)?;
                Ok(t)
            }
            Self::Var(x) => cx.get(x).cloned().wrap_err("Not found: {x}"),
            _ => Err(eyre!("Failed to infer a type for {self}")),
        }
    }

    fn fmt_parens(&self, f: &mut fmt::Formatter<'_>, cond: bool) -> fmt::Result {
        if cond {
            write!(f, "({self})")
        } else {
            write!(f, "{self}")
        }
    }

    fn prec(&self) -> Prec {
        match self {
            Self::Add(_, _) => Prec::Add,
            _ => Prec::Atom,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Add(e_1, e_2) => {
                e_1.fmt_parens(f, self.prec() > e_1.prec())?;
                " + ".fmt(f)?;
                e_2.fmt_parens(f, self.prec() > e_2.prec())
            }
            Self::Command(e) => write!(f, "Command({e})"),
            Self::F64 => "F64".fmt(f),
            Self::F64Lit(n) => n.fmt(f),
            Self::Let(x, e_1, e_2) => write!(f, "let({x}, {e_1}, {e_2})"),
            Self::Trivial => "Trivial".fmt(f),
            Self::Sole => "sole".fmt(f),
            Self::U(i) => write!(f, "U({i})"),
            Self::Var(x) => x.fmt(f),
        }
    }
}

#[derive(Clone)]
pub enum Value {
    Command(Box<Value>),
    F64,
    F64Lit(f64),
    Let(Name, Box<Value>, Box<Value>),
    Trivial,
    Sole,
    U(Level),
}

impl Value {
    pub fn quote(&self) -> Expr {
        match self {
            Self::Command(v) => Expr::Command(Box::new(v.quote())),
            Self::F64 => Expr::F64,
            Self::F64Lit(n) => Expr::F64Lit(*n),
            Self::Let(x, v_1, v_2) => {
                Expr::Let(x.to_owned(), Box::new(v_1.quote()), Box::new(v_2.quote()))
            }
            Self::Trivial => Expr::Trivial,
            Self::Sole => Expr::Sole,
            Self::U(i) => Expr::U(*i),
        }
    }
}

fn get_kinds(r#type: &Type) -> Vec<ExprKind> {
    let mut kinds = vec![ExprKind::Var];

    match r#type {
        Type::Command(t) => match **t {
            Type::Trivial => kinds.push(ExprKind::Let),
            _ => {}
        },
        Type::F64 => kinds.push(ExprKind::F64Lit),
        Type::Trivial => kinds.push(ExprKind::Sole),
        &Type::U(i) if i > 0 => kinds.push(ExprKind::U),
        _ => {}
    }

    kinds
}

#[component]
fn Let(cx: Scope, value: Expr) -> impl IntoView {
    let e_1 = create_rw_signal(cx, ExprKind::Var);

    view! {
        cx,
        <input type="text" placeholder="Name"/>
        // <Expr value=e_1 r#type=Type::U(Level::MAX)/>
        // <Expr value=e_1 r#type=Type::U(Level::MAX)/>
    }
}

#[component]
fn Expr<R, W>(cx: Scope, value: R, set_value: W, r#type: Type) -> impl IntoView
where
    R: Fn() -> Expr,
    W: Fn(Expr),
{
    let kinds = get_kinds(&r#type);
    let initial_kind = kinds[0];
    let kind = create_rw_signal(cx, initial_kind);

    create_effect(cx, move |_| log!("{}", kind()));

    view! {
        cx,
        <select
            on:change=move |ev| {
                let value = event_target_value(&ev);

                let Ok(value) = value.parse() else {
                    return;
                };

                kind.set(value);
            }
        >
            {kinds
                .into_iter()
                .map(|k| view! {
                    cx,
                    <option selected=initial_kind == k>{k}</option>
                })
                .collect::<Vec<_>>()}
        </select>

        {move || match kind() {
            // ExprKind::Add => view! {
            //     cx,
            //     <Expr r#type=Type::F64/>
            //     " + "
            //     <Expr r#type=Type::F64/>
            // }
            // .into_view(cx),
            // ExprKind::Command => view! {
            //     cx,
            //     <Expr r#type=Type::Trivial/>
            // }
            // .into_view(cx),
            // ExprKind::F64 => ().into_view(cx),
            // ExprKind::F64Lit => ().into_view(cx),
            // ExprKind::Let => view! {
            //     cx,
            //     <input type="text" placeholder="Name"/>
            //     <Expr r#type=Type::U(Level::MAX)/>
            // }
            // .into_view(cx),
            ExprKind::Trivial => ().into_view(cx),
            ExprKind::Sole => ().into_view(cx),
            ExprKind::U => ().into_view(cx),
            ExprKind::Var => view! {
                cx,
                <input type="text" placeholder="Name"/>
            }
            .into_view(cx),
            _ => ().into_view(cx),
        }}
    }
}
