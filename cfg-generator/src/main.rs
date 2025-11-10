use anyhow::{Result, Context};
use clap::Parser;
use cfg_generator::{parse_c_code, build_cfg, render_mermaid};
use std::path::Path;
use std::fs;

#[derive(Parser)]
#[command(name = "cfg-gen")]
#[command(about = "C言語の制御フロー図生成ツール")]
struct Cli {
    /// 入力Cファイルまたはディレクトリ（デフォルト: example_code/）
    #[arg(default_value = "example_code")]
    input: String,

    /// 出力ファイルまたはディレクトリ（デフォルト: output/）
    #[arg(short, long, default_value = "output")]
    output: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let input_path = Path::new(&cli.input);
    let output_path = Path::new(&cli.output);

    // 入力がディレクトリの場合
    if input_path.is_dir() {
        // 出力ディレクトリを作成
        fs::create_dir_all(output_path)
            .context("Failed to create output directory")?;

        // ディレクトリ内の.cファイルを処理
        process_directory(input_path, output_path)?;
    } else {
        // 単一ファイルの処理
        process_single_file(input_path, output_path)?;
    }

    Ok(())
}

fn process_directory(input_dir: &Path, output_dir: &Path) -> Result<()> {
    let entries = fs::read_dir(input_dir)
        .context(format!("Failed to read directory: {:?}", input_dir))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // .cファイルのみ処理
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("c") {
            let file_name = path.file_stem()
                .and_then(|s| s.to_str())
                .context("Invalid file name")?;

            let output_file = output_dir.join(format!("{}.md", file_name));

            println!("Processing: {:?} -> {:?}", path, output_file);
            process_single_file(&path, &output_file)?;
        }
    }

    Ok(())
}

fn process_single_file(input_path: &Path, output_path: &Path) -> Result<()> {
    // ファイル読み込み
    let source = fs::read_to_string(input_path)
        .context(format!("Failed to read file: {:?}", input_path))?;

    // パース
    let functions = parse_c_code(&source)?;

    // 各関数のCFG生成
    let mut output = String::new();
    for func in functions {
        let cfg = build_cfg(func)?;
        output.push_str(&render_mermaid(&cfg));
        output.push_str("\n");
    }

    // 出力
    fs::write(output_path, output)
        .context(format!("Failed to write file: {:?}", output_path))?;

    Ok(())
}
