# norenwake

GitHub の自分の公開リポジトリを暖簾分けした新しいリポジトリを作る、Windows 用 TUI 。Rustで書かれています。

## これまでの課題と、このアプリの解決

| これまでの課題 | このアプリの解決 |
| --- | --- |
| 一連の作業の認知負荷が高く、疲労しやすい | TUIで可視化し、認知負荷を軽減します。 |
| 対象 repo を探すのに時間がかかる | `/` の絞り込み（スペース区切り AND）で、対象 repo をすばやく見つけられます。 |
| clone 後に remote が元 repo を向いたままで、誤 push の不安がある | clone 直後に remote を再構成し、`upstream` を削除。`origin` の fetch は HTTPS、push は SSH に統一します。 |
| 新しい repo 名への変更時に設定と README の整合が崩れやすい | `n` で repo 名を変えると、作業ディレクトリ名・remote 設定・`README.ja.md` 先頭ヘッダをまとめて更新します。 |
| 変更内容の確認に時間がかかる | 画面内で README preview と差分（delta）を確認でき、ログも保持されます。 |
| push 前の設定ミスに気づきにくい | `Shift + P` で検証画面を開き、`origin` / push URL / `upstream` をチェック。危険状態ならハードガードで push を拒否します。 |

実運用の最短手順は「`Enter` で clone → `n` で命名 → `c` で commit → `Shift + P` で検証 → `y` で push」です。
一手ずつTUIで進めることで、誤りや漏れのリスクと認知負荷を軽減できます。

## 安全装置

- clone 元として表示するのは「自分の owner リポジトリ」かつ「public / non-fork / non-archived」のみです。
- clone 直後に remote を安全側へ再構成します。
- `upstream` は削除し、`origin` の fetch URL は HTTPS、push URL は SSH に設定します。
- push 前にハードガードを実施します。
- `origin` が暖簾分け元を向いている場合は push を拒否します。
- `upstream` が残っている場合は push を拒否します。

## 必要なもの

- Rust（`cargo`）
- `git`
- `gh`（GitHub CLI）
- `delta`（diffツール）

`gh` は API 認証に使います。token は次の優先順で取得します。

1. `GH_TOKEN`
2. `GITHUB_TOKEN`
3. `gh auth token`

## インストール

```bash
cargo install --force --git https://github.com/cat2151/norenwake
```

## 起動

```bash
norenwake
```

## 更新

```bash
norenwake update
```

## キー操作

- `h` / `l` / `←` / `→`: フォーカスペイン移動（repos / dir tree / log）
- `j` / `k` / `↑` / `↓`: 現在ペイン内で移動
- `PageUp` / `PageDown`: 現在ペイン内をページ移動
- `Enter`（repos ペイン上）: 選択 repo を clone
- `n`: new repo name 編集
- `c`: commit
- `Shift + P`: push 前検証と push confirm
- `Shift + L`: log 全文を clipboard へコピー
- `/`: repos 絞り込み overlay を開く（space 区切り AND 検索）
- `?`: help overlay
- `q`: 終了

## ワークフロー

1. `repos` で対象を選び `Enter` で clone
2. `n` で `new repo name` を編集・確定
3. `c` で commit
4. `Shift + P` で検証結果を確認
5. `y` を押して push

## README 更新ルール

`update_readme_ja` は、`README.ja.md` の先頭に次の暖簾分けヘッダを 1 つだけ維持します。

```md
# <new repo name>

元repoからcloneして暖簾分けしました。暖簾分け断面までの履歴を持っています。
```

既存の同種ヘッダが連続していた場合は畳み込み、重複を残しません。

## README preview

- 取得順は `README.ja.md` 優先、失敗時は `README.md` にフォールバックします。

## データ保存先（Windows）

- `%LOCALAPPDATA%\norenwake\

## 前提
- 自分用のアプリですので、他の人が使うことを想定していません。似たような機能がほしいときは自作をおすすめします。

## このアプリが目指すもの
- PoC。Codexで自分用にあると助かるアプリが作れることを実証する（実証した）

## 目指さないもの（スコープ外）
- サポート。要望や提案に応える
