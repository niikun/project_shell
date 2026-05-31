# 開発ワークフロー

## ブランチ構成

- `develop` — 開発用ブランチ
- `master` — codecrafters 提出用ブランチ（origin は codecrafters リモート）

## 開発からプッシュまでの手順

### 1. develop で開発・コミット

```bash
git add src/main.rs
git commit -m "fix: 変更内容"
```

### 2. master に切り替えて develop をマージ

```bash
git checkout master
git merge develop
```

### 3. codecrafters にプッシュ

```bash
git push origin master
```

## 注意

- codecrafters は `master` ブランチを参照している
- develop の複数コミットを1つにまとめたい場合は `git merge --squash develop` を使う
