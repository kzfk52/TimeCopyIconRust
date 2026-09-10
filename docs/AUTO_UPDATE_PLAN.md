# 自動アップデート導入計画（調査結果・実装前ドキュメント）

**ステータス: 未着手・計画のみ。** このドキュメントはコード変更を伴わない調査結果であり、
[README.md](../README.md) に記載の「今回のスコープ外（次のステップ）」のうち
「GitHub ActionsによるWindows/macOS CI」が完了した**後**に着手する前提でまとめている。

## 結論

Tauri公式の `tauri-plugin-updater` を使えば、**専用の更新サーバーを立てずにGitHub Releasesだけで
自動アップデート環境を構築できる**。GitHub Releasesにビルド成果物と`latest.json`（更新マニフェスト）
を置き、アプリ側がそのURLを定期的に見に行く方式（静的JSON方式）が最も低コスト。

ただし、**macOSでは自動アップデートを実用にするために実質「コード署名+公証(notarization)」が
必須**になる。これは元々「次のステップ」として先送りしていたコード署名/公証を、自動アップデート
導入のタイミングで前倒しする必要があることを意味する。Windowsは未署名でも動作はするが、更新の
たびにSmartScreen警告が出るため推奨はできない。

## 全体像

Tauriのアップデーターは「Tauri独自の署名鍵（minisign方式）」と「OSのコード署名（Authenticode /
Apple Developer ID）」という**2種類の署名**を扱う、別々の仕組みであることに注意。

| 署名の種類 | 目的 | 必須か |
|---|---|---|
| Tauri updater鍵（minisign） | ダウンロードした更新パッケージが改ざんされていないかアプリ自身が検証する | 必須（updater機能を使う以上必ず必要） |
| OSのコード署名（Windows Authenticode / macOS Developer ID + 公証） | OSが「素性の分かるアプリ」として実行を許可する（SmartScreen/Gatekeeper対策） | Windows: 任意（推奨）。macOS: **事実上必須**（無いとGatekeeperがブロックし、更新後のアプリが起動できない/警告だらけになる） |

## 推奨する導入手順（GitHub Actions CI構築の後）

1. **GitHub Actions CIの構築**（既に計画済みの前段）
2. **Tauri updater用の署名鍵ペアを生成**
3. **`tauri-plugin-updater` をアプリに組み込む**（Rust側プラグイン登録 + フロントのUI）
4. **Windowsのコード署名を設定**（任意だが推奨。SmartScreen対策）
5. **macOSのDeveloper ID署名 + 公証(notarization)を設定**（自動アップデートを機能させるには実質必須）
6. **GitHub Actionsで「ビルド→署名→公証→`latest.json`生成→GitHub Releaseへアップロード」を自動化**
7. **アプリ内に「アップデートを確認」のUI/UXを実装**

以下、各ステップの詳細。

---

## 1. Tauri updater鍵ペアの生成

```bash
npm run tauri signer generate -- -w ~/.tauri/timecopyiconrust.key --password "<強いパスワード>"
```

- `-w` で指定した場所に**秘密鍵**が保存され、標準出力に**公開鍵**が表示される。
- **重要な注意点**: `--ci` フラグ（パスワード無しの鍵を生成する簡易オプション）を使うと、
  GitHub Actions上での署名時に「incorrect updater private key password」エラーが発生する事例が
  報告されている（Tauri CLIのバージョンによっては`--ci`生成鍵のKDF構造とCLIの復号処理が噛み合わない
  ため）。**必ずパスワード付きで明示的に生成し、パスワードもSecretに保存する**こと。
- 秘密鍵を紛失/漏洩すると「既存ユーザーへの新規アップデート配信ができなくなる」（漏洩時は鍵の
  ローテーションが必要で、旧鍵で配信済みのユーザーへの更新経路が壊れる）ため、パスワード管理者
  （1Password等）での保管を推奨。

GitHub Secretsに登録するもの:

| Secret名 | 内容 |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | 秘密鍵ファイルの中身（またはパス） |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 鍵生成時に設定したパスワード |

公開鍵は秘密にする必要が無いので、`src-tauri/tauri.conf.json` に直接コミットする。

## 2. `tauri-plugin-updater` の組み込み（このリポジトリでの変更点イメージ）

`src-tauri/Cargo.toml`:
```toml
[dependencies]
tauri-plugin-updater = "2"
```

`src-tauri/src/lib.rs`（他プラグインと同様に登録。デスクトップのみ対応でモバイルには存在しない
プラグインなので `#[cfg(...)]` は不要— updaterプラグイン自体がデスクトップ限定）:
```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

`src-tauri/capabilities/default.json` に権限追加:
```json
"permissions": ["core:default", "updater:default"]
```

`src-tauri/tauri.conf.json`:
```json
{
  "bundle": {
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "pubkey": "<1.で生成した公開鍵の中身>",
      "endpoints": [
        "https://github.com/kzfk52/TimeCopyIconRust/releases/latest/download/latest.json"
      ]
    }
  }
}
```

フロントエンド（`@tauri-apps/plugin-updater` をnpm追加）でのチェック例:
```ts
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

const update = await check();
if (update) {
  await update.downloadAndInstall();
  await relaunch();
}
```

UI設計として、トレイの「Action」メニューに「アップデートを確認」項目を追加し、更新が見つかったら
`status-message` 欄（既存の共有ステータス欄）に案内を出す、という実装が本アプリの既存UIパターンと
自然に噛み合う。

## 3. Windowsのコード署名（任意・推奨）

- OV（Organization Validation）証明書 or EV証明書を購入し、`tauri.conf.json`の
  `bundle.windows.certificateThumbprint`／`digestAlgorithm`／`timestampUrl` を設定するか、
  GitHub Actions上でBase64化した`.pfx`を`WINDOWS_CERTIFICATE`/`WINDOWS_CERTIFICATE_PASSWORD`
  としてSecrets登録し、ビルド時に署名する。
- 未署名でもupdater自体は動作する（インストーラーが実行される）が、更新のたびにSmartScreen警告
  が表示される。EV証明書なら即座に信頼されるが、OV証明書は「レピュテーション」が蓄積するまで
  警告が出ることがある。
- 費用: OV証明書は年間数千〜数万円程度（ベンダーにより変動）。EV証明書はより高額。

## 4. macOSのDeveloper ID署名 + 公証（実質必須）

- **前提**: Apple Developer Program登録（年間 $99）が必要。
- 「Developer ID Application」証明書を作成し、`.p12`としてエクスポート・Base64化して
  `APPLE_CERTIFICATE`/`APPLE_CERTIFICATE_PASSWORD` としてSecrets登録。
- 公証には次のいずれかの認証方式を使う:
  - App Store Connect API方式: `APPLE_API_ISSUER` / `APPLE_API_KEY` / `APPLE_API_KEY_PATH`
  - Apple ID方式: `APPLE_ID` / `APPLE_PASSWORD`（App用パスワード） / `APPLE_TEAM_ID`
- CI用の一時キーチェーンを作るため `KEYCHAIN_PASSWORD` も用意する。
- `tauri-apps/tauri-action`（後述）を使えば、署名・公証・stapling（公証チケットの添付）・
  GitHub Releaseへのアップロードまで一括で面倒を見てくれる。
- **ハマりどころ**: Developer ID証明書の**初回**公証は審査に数時間〜2日程度かかることがある
  （2回目以降は数分程度で完了する例が報告されている）。初回のリリース準備は余裕を持って行う。
- **ad-hoc署名（証明書無し）について**: `signingIdentity`に`-`を指定すると署名自体はできるが、
  Gatekeeperの完全な信頼は得られず、初回起動時にユーザー側で許可操作が必要になる。自動アップデート
  で無人にアプリを差し替える用途には不向き（ユーザーが毎回警告を手動で解除する必要が生じうる）。

## 5. GitHub Actionsでの自動化

公式の `tauri-apps/tauri-action` を使うと、ビルド・（設定していれば）署名・公証・`latest.json`の
生成・GitHub Releaseへのアップロードまでを一括で行ってくれる。ワークフローの骨子（イメージ、要調整）:

```yaml
name: release
on:
  push:
    tags: ["app-v*"]

permissions:
  contents: write

jobs:
  release:
    strategy:
      matrix:
        platform: [windows-latest, macos-latest]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: lts/*
      - uses: dtolnay/rust-toolchain@stable
      - run: npm install
      - uses: tauri-apps/tauri-action@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
          # macOSのみ必要
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
          # Windowsのみ必要（署名する場合）
          WINDOWS_CERTIFICATE: ${{ secrets.WINDOWS_CERTIFICATE }}
          WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
        with:
          tagName: app-v__VERSION__
          releaseName: "TimeCopyIconRust v__VERSION__"
          releaseDraft: true
          prerelease: false
```

補足:
- GitHub Actionsの「Settings → Actions → General → Workflow permissions」で書き込み権限を
  有効にしておく必要がある場合がある（`permissions: contents: write`だけで足りるケースも多いが、
  リポジトリ設定によっては別途有効化が必要）。
- `tagName`/`releaseName`の`__VERSION__`は`tauri.conf.json`の`version`から自動置換される。
- `releaseDraft: true`にしておけば、Releaseが自動で下書き作成され、内容確認後に手動で公開する
  運用にできる（誤って壊れたビルドを配信するリスクを下げられる）。
- endpointは`.../releases/latest/download/latest.json`という固定URLなので、Release公開時に
  この名前で`latest.json`が置かれていれば、タグ名が変わってもアプリ側の設定変更は不要。

## 6. アプリ内UI/UX（実装時の検討事項）

- 起動時に自動チェックするか、メニューから手動チェックにするかは要検討。バックグラウンド常駐
  アプリという性質上、**起動時の自動チェック＋メニューからの手動チェックの併用**が妥当と思われる。
- ダウンロード進捗・エラー時のフォールバック（ネットワーク不通、署名検証失敗等）を
  既存の共有ステータス欄（`status-message`）に出す設計は、AnnounceP等と同じ実装パターンを流用
  できる。
- 更新適用後は`relaunch()`でアプリを再起動する必要がある（`@tauri-apps/plugin-process`）。

## 未確定・要検討事項

- Windowsのコード署名証明書（OV/EV）を実際に購入するかどうか、費用対効果の判断はユーザー側の
  意思決定が必要。
- Apple Developer Program登録（年間$99）を行うかどうかも同様。**これを行わない場合、macOS版の
  自動アップデートは事実上実用にならない**（配布自体は可能だが、初回起動時の警告解除をユーザーに
  都度求めることになる）。
- 更新チェックの頻度・タイミング、ロールバック方針（不具合のあるバージョンを配信してしまった
  場合の対応）は別途検討が必要。

## 参考資料

- [Tauri v2 公式ドキュメント: Updater Plugin](https://v2.tauri.app/plugin/updater/)
- [Tauri v2 公式ドキュメント: Distribute via GitHub](https://v2.tauri.app/distribute/pipelines/github/)
- [Tauri v2 公式ドキュメント: Sign macOS applications](https://v2.tauri.app/distribute/sign/macos/)
- [Tauri v2 公式ドキュメント: Sign Windows applications](https://v2.tauri.app/distribute/sign/windows/)
- [Tauri製macOSアプリのコード署名・公証・自動アップデート署名を自動化し、GitHub Releasesで配信する（DevelopersIO）](https://dev.classmethod.jp/articles/shuntaka-tauri-macos-codesign-notarization-updater-github-releases/)
- [Whether macOS Notarization would affect updater signature? (tauri-apps/tauri Discussion #7703)](https://github.com/tauri-apps/tauri/discussions/7703)
