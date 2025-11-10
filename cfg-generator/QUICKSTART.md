# クイックスタート

## 最速で試す（3ステップ）

### 1. ビルド

```bash
cd cfg-generator
cargo build --release
```

### 2. 実行

```bash
./target/release/cfg-generator
```

### 3. 結果を確認

```bash
cat output/example.md
```

これで `example_code/` 内の全てのCファイルから制御フロー図が生成されます！

---

## 詳細な実行例

### 例1: デフォルト設定で実行

```bash
$ ./target/release/cfg-generator
Processing: "example_code/example.c" -> "output/example.md"
Processing: "example_code/test_complex.c" -> "output/test_complex.md"
```

**生成されるファイル:**
- `output/example.md`
- `output/test_complex.md`

### 例2: 自分のCファイルを処理

1. Cファイルを `example_code/` に配置:
```bash
cp my_program.c example_code/
```

2. ツールを実行:
```bash
./target/release/cfg-generator
```

3. 結果を確認:
```bash
ls output/
# my_program.md が生成されている
```

### 例3: 別のディレクトリから入力

```bash
./target/release/cfg-generator /path/to/my/c/files -o my_output
```

### 例4: 単一ファイルのみ処理

```bash
./target/release/cfg-generator example_code/example.c -o result.md
```

---

## 生成されたMermaid図の見方

### ノードの種類

| 形状 | 意味 | 例 |
|------|------|-----|
| `([...])` | 開始/終了 | `([START: main])` |
| `{...}` | 条件分岐 | `{x > 0}` |
| `[...]` | 処理文 | `["return 1;"]` |

### エッジのラベル

- `-->|true|` : 条件が真の場合
- `-->|false|` : 条件が偽の場合
- `-->` : 通常の制御フロー

---

## 出力をGitHubで確認

生成された `.md` ファイルをGitHubにpushすると、自動的にMermaid図がレンダリングされます。

```bash
git add output/*.md
git commit -m "Add control flow graphs"
git push
```

GitHubのWebインターフェースでファイルを開くと、図が表示されます！

---

## よくある使い方

### パターン1: プロジェクト全体を解析

```bash
# プロジェクトのsrc/ディレクトリを解析
./target/release/cfg-generator ../my_project/src -o docs/cfg
```

### パターン2: CI/CDで自動生成

```yaml
# .github/workflows/cfg.yml
name: Generate CFG
on: [push]
jobs:
  cfg:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cd cfg-generator && cargo build --release
      - run: ./cfg-generator/target/release/cfg-generator src -o docs/cfg
      - uses: stefanzweifel/git-auto-commit-action@v4
        with:
          commit_message: Update CFG
```

### パターン3: 特定の関数だけ解析したい場合

現在は自動で全関数を解析しますが、特定の関数だけ抽出したい場合は：

```bash
# example.cのみを処理
./target/release/cfg-generator example_code/example.c -o output/specific.md
```

---

## 次のステップ

- `example_code/` に自分のCファイルを追加してみる
- 生成された図をドキュメントに組み込む
- 複雑なif文のネストを可視化してみる

詳細はREADME.mdを参照してください。
