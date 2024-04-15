# keyball-embassy-rp2040

ProMicro RP2040用のRust(+Embassy)製Keyballファームウェアです。

まだまだwipです。とりあえず何かしら文字は打てます。

## ビルド

1. `elf2uf2-rs`をインストール

```
cargo install elf2uf2-rs
```

2. BOOTSELボタンを押しながらProMicro RP2040をUSBに接続
3. 実行

```
cargo run --release
```

※このファームにはダブルリセットでBOOTSELに入る機能が内蔵されているため、以降はBOOTSELボタンを押しながら差す必要はない

## 進行状況

### 基本機能

- [x] 基本的なキースキャンとUSB HIDへの出力
- [x] PMW3360によるマウス入力とUSB HIDへの出力
- [ ] 分割キーボード間通信(partial)
  - [x] 半二重通信によるバイト列の送受信
  - [x] rkyvによるバイト列のシリアライズ/デシリアライズ
  - [ ] master側でのデータの受信と処理
- [ ] OLEDディスプレイ
  - [x] 文字表示
  - [ ] ステータス表示
  - [ ] 画像表示
  - [ ] せっかく容量があるのでアニメーションとか表示したい
- [ ] LED(ws2812)
  - [x] とりあえずなんか光る
  - [ ] きれいに光らせる

### やらなければならないこと

- [ ] master/slave判定をちゃんとする(今はマウスあるかどうかで判定している)
- [ ] PMW3360のSPI通信の安定化(なぜか認識されないことが多々ある)
- [ ] 分割キーボード間通信の安定化(何回かリセットしないとうまく動かないことがある)

### やりたいこと

- [ ] 高度なキーマップ機構(レイヤなど)
- [ ] 左トラックボール対応

### 将来の展望

- [ ] Keyball61以外の対応
- [ ] VIA対応
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
