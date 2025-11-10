use anyhow::{Result, Context};
use tree_sitter::{Parser, Node};

pub struct Function {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Simple {
        code: String,
        line: usize,
    },
    If {
        condition: String,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
        line: usize,
    },
}

pub fn parse_c_code(source: &str) -> Result<Vec<Function>> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_c::language())
        .context("Failed to load C grammar")?;

    let tree = parser.parse(source, None)
        .context("Failed to parse source code")?;

    let root = tree.root_node();
    let mut functions = Vec::new();

    // 関数定義を探す
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if child.kind() == "function_definition" {
            if let Some(func) = parse_function(child, source) {
                functions.push(func);
            }
        }
    }

    Ok(functions)
}

fn parse_function(node: Node, source: &str) -> Option<Function> {
    // 関数名を取得
    let name = get_function_name(node, source)?;

    // 関数本体を取得
    let body_node = node.child_by_field_name("body")?;
    let body = parse_compound_statement(body_node, source);

    Some(Function { name, body })
}

fn get_function_name(node: Node, source: &str) -> Option<String> {
    let declarator = node.child_by_field_name("declarator")?;
    let identifier = find_identifier(declarator)?;
    Some(source[identifier.byte_range()].to_string())
}

fn find_identifier(node: Node) -> Option<Node> {
    if node.kind() == "identifier" {
        return Some(node);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(id) = find_identifier(child) {
            return Some(id);
        }
    }
    None
}

fn parse_compound_statement(node: Node, source: &str) -> Vec<Statement> {
    let mut statements = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            "if_statement" => {
                if let Some(stmt) = parse_if_statement(child, source) {
                    statements.push(stmt);
                }
            }
            "expression_statement" |
            "declaration" |
            "return_statement" => {
                let code = source[child.byte_range()].trim().to_string();
                let line = child.start_position().row + 1;
                statements.push(Statement::Simple { code, line });
            }
            "compound_statement" => {
                // ネストしたブロック
                let nested = parse_compound_statement(child, source);
                statements.extend(nested);
            }
            _ => {}
        }
    }

    statements
}

fn parse_if_statement(node: Node, source: &str) -> Option<Statement> {
    let condition_node = node.child_by_field_name("condition")?;
    // 括弧を除去: parenthesized_expressionの中身を取得
    let condition = if condition_node.kind() == "parenthesized_expression" {
        let mut cursor = condition_node.walk();
        let children: Vec<_> = condition_node.children(&mut cursor).collect();
        // 括弧の間の内容を取得（最初と最後は括弧）
        if children.len() >= 3 {
            source[children[1].byte_range()].to_string()
        } else {
            source[condition_node.byte_range()].to_string()
        }
    } else {
        source[condition_node.byte_range()].to_string()
    };

    let consequence = node.child_by_field_name("consequence")?;
    let then_branch = if consequence.kind() == "compound_statement" {
        parse_compound_statement(consequence, source)
    } else {
        vec![Statement::Simple {
            code: source[consequence.byte_range()].to_string(),
            line: consequence.start_position().row + 1,
        }]
    };

    let else_branch = node.child_by_field_name("alternative").map(|alt| {
        // alternativeは"else_clause"なので、その中のstatementを探す
        if alt.kind() == "else_clause" {
            // else_clauseの子ノードから実際のstatementを取得
            let mut cursor = alt.walk();
            for child in alt.children(&mut cursor) {
                match child.kind() {
                    "compound_statement" => {
                        return parse_compound_statement(child, source);
                    }
                    "if_statement" => {
                        return vec![parse_if_statement(child, source).unwrap()];
                    }
                    _ => {}
                }
            }
            vec![]
        } else if alt.kind() == "compound_statement" {
            parse_compound_statement(alt, source)
        } else if alt.kind() == "if_statement" {
            // else if の場合
            vec![parse_if_statement(alt, source).unwrap()]
        } else {
            vec![Statement::Simple {
                code: source[alt.byte_range()].to_string(),
                line: alt.start_position().row + 1,
            }]
        }
    });

    Some(Statement::If {
        condition,
        then_branch,
        else_branch,
        line: node.start_position().row + 1,
    })
}
