#[derive(Debug, Clone, PartialEq)]
pub enum DocCore {
    Nil,
    Append(Box<DocCore>, Box<DocCore>),
    Nest(i64, Box<DocCore>),
    Text(String),
    Line,
    Union(Box<DocCore>, Box<DocCore>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Doc {
    Nil,
    Text(String, Box<Doc>),
    Line(i64, Box<Doc>),
}

// TODO: move to pretty module.

pub fn nil() -> DocCore {
    DocCore::Nil
}

// append is right associative.
pub fn append(x: DocCore, y: DocCore) -> DocCore {
    DocCore::Append(x.into(), y.into())
}

pub fn nest(i: i64, x: DocCore) -> DocCore {
    DocCore::Nest(i, x.into())
}

pub fn text(s: String) -> DocCore {
    DocCore::Text(s)
}

pub fn line() -> DocCore {
    DocCore::Line
}

pub fn group(x: DocCore) -> DocCore {
    DocCore::Union(flatten(x.clone()).into(), x.clone().into())
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

pub fn copy(i: i64, x: &str) -> String {
    std::iter::repeat(x)
        .take(i as usize)
        .collect::<Vec<_>>()
        .join("")
}

pub fn best(w: i64, k: i64, x: DocCore) -> Doc {
    be(w, k, &[(0, x)])
}

pub fn be(w: i64, k: i64, xs: &[(i64, DocCore)]) -> Doc {
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
        Some(((_i, Text(s)), z)) => Doc::Text(s.clone(), be(w, k + s.len() as i64, z).into()),
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

pub fn better(w: i64, k: i64, x: Doc, y: Doc) -> Doc {
    if fits(w - k, x.clone()) {
        x
    } else {
        y
    }
}

pub fn fits(w: i64, x: Doc) -> bool {
    // NOTE: if we were using isize we'd keep this condition.
    if w < 0 {
        return false;
    }
    use Doc::*;
    match x {
        Nil => true,
        Text(s, x) => fits(w - s.len() as i64, *x.clone()),
        Line(_i, _x) => true,
    }
}

pub fn pretty(w: i64, x: DocCore) -> String {
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

// TODO: examples.

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    pub enum Tree {
        Node(String, Vec<Box<Tree>>),
    }

    pub fn show_tree(tree: Tree) -> DocCore {
        match tree {
            Tree::Node(s, ts) => group(append(
                text(s.clone()),
                nest(s.len() as i64, show_bracket(&ts)),
            )),
        }
    }

    pub fn show_bracket(ts: &[Box<Tree>]) -> DocCore {
        if ts.is_empty() {
            nil()
        } else {
            append(
                text(String::from("[")),
                append(nest(1, show_trees(ts)), text(String::from("]"))),
            )
        }
    }

    pub fn show_trees(ts: &[Box<Tree>]) -> DocCore {
        match ts.split_first() {
            None => unreachable!("it's fun to dream"),
            Some((t, &[])) => show_tree(*(*t).clone()),
            Some((t, ts)) => append(
                show_tree(*(*t).clone()),
                append(text(String::from(",")), append(line(), show_trees(&ts))),
            ),
        }
    }

    pub fn show_tree_prime(tree: Tree) -> DocCore {
        match tree {
            Tree::Node(s, ts) => append(text(s.clone()), show_bracket_prime(&ts)),
        }
    }

    pub fn show_bracket_prime(ts: &[Box<Tree>]) -> DocCore {
        if ts.is_empty() {
            nil()
        } else {
            bracket(String::from("["), show_trees_prime(&ts), String::from("]"))
        }
    }

    pub fn show_trees_prime(ts: &[Box<Tree>]) -> DocCore {
        match ts.split_first() {
            None => unreachable!("it's fun to dream"),
            Some((t, &[])) => show_tree(*(*t).clone()),
            Some((t, ts)) => append(
                show_tree(*(*t).clone()),
                append(text(String::from(",")), append(line(), show_trees(&ts))),
            ),
        }
    }

    fn tree() -> Tree {
        Tree::Node(
            String::from("aaa"),
            vec![
                Tree::Node(
                    String::from("bbbbb"),
                    vec![
                        Tree::Node(String::from("ccc"), vec![]).into(),
                        Tree::Node(String::from("dd"), vec![]).into(),
                    ],
                )
                .into(),
                Tree::Node(String::from("eee"), vec![]).into(),
                Tree::Node(
                    String::from("ffff"),
                    vec![
                        Tree::Node(String::from("gg"), vec![]).into(),
                        Tree::Node(String::from("hhh"), vec![]).into(),
                        Tree::Node(String::from("ii"), vec![]).into(),
                    ],
                )
                .into(),
            ],
        )
    }

    #[test]
    fn show_tree_01() {
        insta::assert_snapshot!(pretty(30, show_tree(tree())));
    }

    #[test]
    fn show_tree_02() {
        insta::assert_snapshot!(pretty(30, show_tree_prime(tree())));
    }
}
