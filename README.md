# TimeCopyIconRust

[TimeCopyIconWinForms](../TimeCopyIconWinForms)（C# / WinForms）の Tauri (Rust + TypeScript) 移植版。
Windows / macOS で動作するトレイ常駐型の時刻・日付変換ユーティリティです。

## 機能

- トレイアイコン常駐。ダブルクリック（Windowsのみ）または右クリックメニューから現在時刻をクリップボードへコピー
  - UnixTime値をコピー
  - `Y/m/d H:i:s` 形式でコピー
  - `YmdHis` 形式でコピー
- 「AnnounceP」: クリップボードのテキストの改行を `<br/>` に置換し `<p>...</p>` で囲んで書き戻す
- ウィンドウ内フォーム
  - unixtime → ローカル日時文字列への変換
  - ISO 8601 等の日付文字列 → unixtime への変換
- 多重起動防止（既存ウィンドウを表示してフォーカス）
- ウィンドウ位置・サイズ・最大化状態を次回起動時にも記憶（原作には無い追加機能）

移植元との仕様差分・既知の制約は [docs/PORTING_NOTES.md](docs/PORTING_NOTES.md) を参照してください。

## 開発

前提: Node.js, Rust, `@tauri-apps/cli` (`cargo tauri` / `npm run tauri`)。

```bash
npm install
npm run app:dev
```

Rust側のロジック（日時変換）の単体テスト:

```bash
cd src-tauri
cargo test
```

## ビルド

インストーラー（Windows: msi/nsis、macOS: dmg/app バンドル）まで作る場合:

```bash
npm run tauri build
```

インストーラー生成をスキップし、実行バイナリのみをビルドする場合（Windowsは
`.exe`、macOSは`.app`/実行ファイルが `src-tauri/target/release/` 配下に出力される。
GitHub Actions等、各OSのランナー上でネイティブビルドする用途を想定):

```bash
npm run app:build
```

いずれもクロスコンパイルではなく、実行しているOS向けのバイナリが生成される点に注意
（例: Linux上で実行してもWindows/macOS向けバイナリは作れない）。Windows/macOS向けの
実バイナリは、各OS上（または後続のGitHub Actionsのwindows-latest/macos-latestランナー
上）で実行する必要がある。

Windows/macOS向けのアイコンは `src-tauri/icons/` に、移植元の `stopwatch.ico` から
`cargo tauri icon` で生成済みです。

## 今回のスコープ外（次のステップ）

- コード署名（Windows Authenticode / macOS codesign）
- macOS 公証（notarization）
- GitHub Actions による Windows/macOS クロスプラットフォームCI
- 自動アップデート（GitHub Actions CI構築後に着手。調査結果・導入手順は
  [docs/AUTO_UPDATE_PLAN.md](docs/AUTO_UPDATE_PLAN.md) を参照）

## License

Copyright © 2023, [FUKUDA Kazuyuki](https://github.com/kzfk).
Released under the [MIT License](LICENSE).
