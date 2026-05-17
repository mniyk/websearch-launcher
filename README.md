# WebSearch Launcher

Alfred / Raycast のようなランチャー型の Web 検索 + AI 要約デスクトップアプリです。
キーワードを入力すると Tavily Search API で検索し、ローカルで動作する Ollama LLM が日本語で要約を生成します。

## 必要なもの

- [Rust](https://www.rust-lang.org/tools/install) (1.75 以上)
- [Ollama](https://ollama.com/) (ローカル LLM ランタイム)
- [Tavily API キー](https://tavily.com/) (無料プランあり)

## セットアップ手順

### 1. Ollama のセットアップ

```bash
# Ollama をインストール後、使用するモデルを取得
ollama pull qwen2.5:7b
```

### 2. リポジトリのクローン

```bash
git clone <repo-url>
cd websearch-launcher
```

### 3. ビルドと起動

```bash
cargo run --release
```

## 使い方

1. アプリを起動すると小窓が表示されます（always-on-top）
2. 検索欄にキーワードを入力して **Enter** キーまたは「検索」ボタンを押します
3. Tavily で Web 検索 → Ollama で日本語要約が生成されます
4. 要約の下に**参照元 URL** が表示され、クリックするとブラウザで開きます
5. 「最新ニュース寄りで検索する」チェックを入れると、Tavily の `topic=news` モードで検索します

## 設定画面

右上の **⚙** アイコンから設定画面を開けます。

| 項目 | デフォルト | 説明 |
|------|-----------|------|
| Tavily API キー | （空） | [tavily.com](https://tavily.com/) で取得。空欄時はシステム環境変数 `TAVILY_API_KEY` を使用 |
| Ollama ホスト | `http://localhost:11434` | Ollama サーバーの URL。別マシンの場合は IP を指定 |
| モデル名 | `qwen2.5:7b` | Ollama で使用するモデル名 |

設定は OS の設定ディレクトリ（Windows: `%APPDATA%\websearch-launcher\settings.json`）に保存されます。

## エラーが出た場合

| エラーメッセージ | 原因と対処 |
|---------------|-----------|
| `TAVILY_API_KEY が設定されていません` | 設定画面で Tavily API キーを入力してください |
| `検索結果が 0 件でした` | キーワードを変えて試してください |
| `Ollama に接続できません` | `ollama serve` が起動しているか確認してください |
| `Tavily API エラー (401)` | API キーが無効です |
