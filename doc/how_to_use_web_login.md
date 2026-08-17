# Web版でログインする（実験的）

櫻坂46・日向坂46・乃木坂46の公式Web版から一時的なアクセストークンを取得して `colmsg` を実行します。`refresh_token` は取得・保存しません。Google Chromeが必要ですが、Node.jsなどの追加ランタイムは必要ありません。

```shell
colmsg --web-login-setup --group nogizaka
```

初回は上のコマンドで開いたChrome上で通常どおりログインし、Web版が表示されたらChromeを閉じてください。Googleログインはデバッグ用のブラウザを拒否するため、この初回設定は通常のChromeで行います。

設定後は次のコマンドで `colmsg` を実行します。専用ブラウザプロファイルにログイン状態が残るため、通常は初回設定の再実行は必要ありません。

```shell
colmsg --web-login --group nogizaka
```

対象は `sakurazaka`、`hinatazaka`、`nogizaka` から1つを `--group` で指定します。通常のオプションも同時に使えます。

```shell
colmsg --web-login --group hinatazaka -k picture -k video
```

Chromeが自動検出されない場合は、`COLMSG_CHROME` にChrome実行ファイルのパスを設定してください。ブラウザプロファイルの場所は `COLMSG_WEB_PROFILE` で変更できます。このディレクトリにはログイン状態が保存されるため、他人と共有しないでください。

Web版のアクセストークンは短時間で期限切れになります。ダウンロード中に期限切れになった場合は、同じコマンドを再実行してください。保存済みのメッセージは再ダウンロードされません。
