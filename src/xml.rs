use crate::errors::*;

use roxmltree::Node;

pub fn get_child<'a, 'b>(node: &Node<'a, 'b>, name: &str) -> Result<Node<'a, 'b>> {
    node.children()
        .filter(|child| child.has_tag_name(name))
        .nth(0)
        .ok_or_else(|| {
            format!(
                "Did not find child {} under node {}",
                name,
                node.tag_name().name()
            )
            .into()
        })
}

pub fn get_child_text<'a, 'b>(node: &Node<'a, 'b>, name: &str) -> Result<&'a str> {
    get_child(node, name)?
        .text()
        .ok_or_else(|| format!("Node {} does not contain any text", name).into())
}

pub fn get_attrib<'a, 'b>(node: &Node<'a, 'b>, name: &'a str) -> Result<&'a str> {
    node.attribute(name).ok_or_else(|| {
        format!(
            "Did not find attribute {} under node {}",
            name,
            node.tag_name().name()
        )
        .into()
    })
}
