#[derive(Debug, Clone, PartialEq)]
pub enum DocCore {
    Nil,
    Append(Box<DocCore>, Box<DocCore>),
    Nest(usize, Box<DocCore>),
    Text(String),
    Line,
    Union(Box<DocCore>, Box<DocCore>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Doc {
    Nil,
    Text(String, Box<Doc>),
    Line(usize, Box<Doc>),
}

// TODO: move to pretty module.

pub fn nil() -> DocCore {
    DocCore::Nil
}

// append is right associative.
pub fn append(x: DocCore, y: DocCore) -> DocCore {
    DocCore::Append(x.into(), y.into())
}

pub fn nest(i: usize, x: DocCore) -> DocCore {
    DocCore::Nest(i, x.into())
}

pub fn text(s: String) -> DocCore {
    DocCore::Text(s)
}

pub fn line() -> DocCore {
    DocCore::Line
}

pub fn group(x: DocCore) -> DocCore {
    flatten(DocCore::Append(x.clone().into(), x.clone().into()))
}

pub fn flatten(x: DocCore) -> DocCore {
    use DocCore::*;
    match x {
        Nil => Nil,
        Append(x, y) => Append(flatten(*x).into(), flatten(*y).into()),
        Nest(i, x) => Nest(i, flatten(*x).into()),
        Text(s) => Text(s),
        Line => Text(String::from(" ")),
        Union(x, _y) => flatten(*x),
    }
}

pub fn layout(x: Doc) -> String {
    use Doc::*;
    match x {
        Nil => String::from(""),
        Text(s, x) => format!("{}{}", s, layout(*x)),
        Line(i, x) => format!("\n{}{}", copy(i, " "), layout(*x)),
    }
}

pub fn copy(i: usize, x: &str) -> String {
    std::iter::repeat(x).take(i).collect::<Vec<_>>().join("")
}

pub fn best(w: usize, k: usize, x: DocCore) -> Doc {
    be(w, k, &[(0, x)])
}

pub fn be(w: usize, k: usize, xs: &[(usize, DocCore)]) -> Doc {
    use DocCore::*;
    match xs.split_first() {
        None => Doc::Nil,
        Some(((_i, Nil), z)) => be(w, k, &z),
        Some(((i, Append(x, y)), z)) => {
            let mut zs = vec![(*i, *x.clone()), (*i, *y.clone())];
            zs.extend_from_slice(z);
            be(w, k, &zs)
        }
        Some(((i, Nest(j, x)), z)) => {
            let mut zs = vec![(i + j, *x.clone())];
            zs.extend_from_slice(z);
            be(w, k, &zs)
        }
        Some(((_i, Text(s)), z)) => Doc::Text(s.clone(), be(w, k + s.len(), z).into()),
        Some(((i, Line), z)) => Doc::Line(*i, be(w, *i, z).into()),
        Some(((i, Union(x, y)), z)) => {
            let mut zs1 = vec![(*i, *x.clone())];
            let mut zs2 = vec![(*i, *y.clone())];
            zs1.extend_from_slice(z);
            zs2.extend_from_slice(z);
            better(w, k, be(w, k, &zs1), be(w, k, &zs2))
        }
    }
}

pub fn better(w: usize, k: usize, x: Doc, y: Doc) -> Doc {
    if fits(w - k, x.clone()) {
        x
    } else {
        y
    }
}

pub fn fits(w: usize, x: Doc) -> bool {
    // NOTE: if we were using isize we'd keep this condition.
    //if w < 0 {
    //    return false;
    //}
    use Doc::*;
    match x {
        Nil => true,
        Text(s, x) => fits(w - s.len(), *x.clone()),
        Line(_i, _x) => true,
    }
}

pub fn pretty(w: usize, x: DocCore) -> String {
    layout(best(w, 0, x))
}

// TODO: move to utilities module.

pub fn space(x: DocCore, y: DocCore) -> DocCore {
    append(x, append(text(String::from(" ")), y))
}

pub fn newline(x: DocCore, y: DocCore) -> DocCore {
    append(x, append(line(), y))
}

pub fn fold_doc<F>(f: &F, xs: &[DocCore]) -> DocCore
where
    F: Fn(DocCore, DocCore) -> DocCore,
{
    match xs.split_first() {
        None => nil(),
        Some((x, &[])) => x.clone(),
        Some((x, xs)) => f(x.clone(), fold_doc(f, xs)),
    }
}

pub fn spread(xs: &[DocCore]) -> DocCore {
    fold_doc(&space, xs)
}

pub fn stack(xs: &[DocCore]) -> DocCore {
    fold_doc(&newline, xs)
}

pub fn bracket(l: String, x: DocCore, r: String) -> DocCore {
    group(append(
        text(l),
        append(nest(2, append(line(), x)), append(line(), text(r))),
    ))
}

pub fn space_newline(x: DocCore, y: DocCore) -> DocCore {
    append(x, append(append(text(String::from(" ")), line()), y))
}

pub fn fill_words(s: String) -> DocCore {
    fold_doc(
        &space_newline,
        &s.split(" ")
            .map(|x| text(String::from(x)))
            .collect::<Vec<_>>(),
    )
}

pub fn fill(xs: &[DocCore]) -> DocCore {
    match &xs {
        &[] => nil(),
        &[x] => x.clone(),
        &[x, y, zs @ ..] => {
            let mut plain_zs = vec![y.clone()];
            let mut flattened_zs = vec![flatten(y.clone())];
            plain_zs.extend_from_slice(zs);
            flattened_zs.extend_from_slice(zs);
            append(
                space(flatten(x.clone()), fill(&flattened_zs)),
                newline(x.clone(), fill(&plain_zs)),
            )
        }
    }
}
