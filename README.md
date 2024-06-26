> [!IMPORTANT]
>
> リポジトリを移動しました。
>
> https://github.com/nazo6/rktk/tree/master/keyboards/keyball61-rp2040

# keyball-embassy-rp2040

ProMicro RP2040用のRust(+Embassy)製Keyballファームウェアです。

紹介記事:
[RustとEmbassyでKeyballのファームウェアを作った](https://zenn.dev/nazo6/articles/keyball-embassy-rp2040)

## ステータス

基本的なキースキャン機能とレイヤ機能を実装しています。`keyboard/keymap.rs`を編集することでキーマップを変更できます。

## 既知の不具合

- 左右間の通信が不安定なことがある
- メディアキーが効かない

## ビルド

1. `elf2uf2-rs`をインストール

```
cargo install elf2uf2-rs
```

2. BOOTSELボタンを押しながらProMicro RP2040をUSBに接続
3. `src/config.example.rs`を`src/config.rs`にコピー
4. `src/keyboard/keymap.example.rs`を`src/keyboard/keymap.rs`にコピー
5. 実行

```
cargo run --release
```

> [!TIP]
> このファームにはダブルリセットでBOOTSELに入る機能が内蔵されているため、以降はBOOTSELボタンを押しながら差す必要はありません。

> [!NOTE]
> 通常のProMicroのようにハードウェアでダブルリセットを検知しているわけではないので、ダブルタップが早すぎると検知されません。

## 進行状況

### 基本機能

- [x] 基本的なキースキャンとUSB HIDへの出力
- [x] PMW3360によるマウス入力とUSB HIDへの出力
- [x] 分割キーボード間通信(partial)
  - [x] 半二重通信によるバイト列の送受信
  - [x] postcardによるバイト列のシリアライズ/デシリアライズ
  - [x] master側でのデータの受信と処理
- [ ] OLEDディスプレイ
  - [x] 文字表示
  - [x] ステータス表示
  - [ ] 画像表示
  - [ ] せっかく容量があるのでアニメーションとか表示したい
- [ ] LED(ws2812)
  - [x] とりあえずなんか光る
  - [ ] きれいに光らせる

### やらなければならないこと

- [x] master/slave判定をちゃんとする(今はマウスあるかどうかで判定している)
- [x] PMW3360のSPI通信の安定化(なぜか認識されないことが多々ある)
- [ ] sendの失敗検知と再送信
- [x] OLED未接続時にpanicしないようにする
- [ ] master側がまあまあ熱くなるのを直す

### やりたいこと

- [ ] 高度なキーマップ機構(レイヤなど)
- [x] 左トラックボール対応

### 将来の展望

- [ ] Keyball61以外の対応
- [ ] Vial対応
- [ ] BLE Micro Pro対応(NRF52なのでできないことはなさそう？)

## CREDITS

これらの先人がいなければここまで作れませんでした。

- もろもろ参考
  - [keyball-rs](https://github.com/hikalium/keyball-rs)
- ダブルリセットでBOOTSEL
  - https://github.com/Univa/rumcake/blob/2fa47dce9ab2b2406dd5465ccc9ce7b23e5ffdb0/rumcake/src/hw/mod.rs
- PMW3360ドライバ関連
  - https://github.com/kndndrj/mouse/tree/8c3cf4707cc392c16c91dc11e53f954f0fd820f1/firmware-rust/mouse-libs/src/pmw3360
- 分割キーボード間半二重通信
  - [QMK Firmware](https://github.com/qmk/qmk_firmware/blob/master/platforms/chibios/drivers/vendor/RP/RP2040/serial_vendor.c)
  - [rusty-dilemma](https://github.com/simmsb/rusty-dilemma/blob/5ffe8f5d2b6b0d534a4309edc737364cd96f44f1/firmware/src/interboard/onewire.rs)
