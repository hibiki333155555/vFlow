use cfg_generator::{parse_c_code, build_cfg, render_mermaid};

#[test]
fn test_simple_if() {
    let source = r#"
        int max(int a, int b) {
            if (a > b) {
                return a;
            } else {
                return b;
            }
        }
    "#;

    let functions = parse_c_code(source).unwrap();
    assert_eq!(functions.len(), 1);

    let cfg = build_cfg(functions.into_iter().next().unwrap()).unwrap();
    let output = render_mermaid(&cfg);

    assert!(output.contains("START: max"));
    assert!(output.contains("END: max"));
    assert!(output.contains("a &gt; b") || output.contains("a > b"));
    assert!(output.contains("true"));
    assert!(output.contains("false"));
}

#[test]
fn test_sequential_statements() {
    let source = r#"
        void process() {
            int x = 10;
            x = x + 1;
            printf("%d\n", x);
        }
    "#;

    let functions = parse_c_code(source).unwrap();
    let cfg = build_cfg(functions.into_iter().next().unwrap()).unwrap();
    let output = render_mermaid(&cfg);

    println!("{}", output); // デバッグ用出力

    assert!(output.contains("int x = 10"));
    assert!(output.contains("x = x + 1"));
    assert!(output.contains("printf"));
}
