use comrak::{Arena, parse_document, ComrakOptions, nodes::{NodeValue, AstNode, NodeCode}};

pub fn get_title(document: &str) -> String {
    let arena = Arena::new();
    let root = parse_document(&arena, document, &ComrakOptions::default()); 
    for node in root.children() {
        let header = match node.data.clone().into_inner().value {
            NodeValue::Heading(head) => head,
            _ => continue
        };
        if header.level != 1 {
            continue;
        }
        let mut text = Vec::new();
        collect_text(node, &mut text);
        
        return String::from_utf8(text).unwrap();
    } 
    "untitled document".to_string()
}
fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut Vec<u8>) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) | NodeValue::Code(NodeCode { ref literal, .. }) => {
            output.extend_from_slice(literal)
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
        _ => {
            for n in node.children() {
                collect_text(n, output);
            }
        }
    }
}
