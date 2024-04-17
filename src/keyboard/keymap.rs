use super::keycode::key::Key::*;
use super::keycode::Keycode::*;
use super::keycode::*;
use crate::constant::{COLS, ROWS};

#[rustfmt::skip]
pub const KEYMAP: [[Keycode; COLS * 2]; ROWS] = [
    [____, Key(D1), Key(D2), Key(D3), Key(D4), Key(D5), ____ , ____ , Key(D6), Key(D7), Key(D8)   , Key(D9) , Key(D0), ____],
    [____, Key(Q) , Key(W) , Key(E) , Key(R) , Key(T) , ____ , ____ , Key(Y) , Key(U) , Key(I)    , Key(O)  , Key(P), ____],
    [____, Key(A) , Key(S) , Key(D) , Key(F) , Key(G) , ____ , ____ , Key(H) , Key(J) , Key(K)    , Key(L)  , Key(Semicolon), ____],
    [____, Key(Z) , Key(X) , Key(C) , Key(V) , Key(B) , ____ , ____ , Key(N) , Key(M) , Key(Comma), Key(Dot), Key(Slash), ____],
    [____, ____   , Key(X) , Key(C) , Key(V) , Key(B) , ____ , ____ , Key(N) , Key(M) , Key(Comma), Key(Dot), Key(Slash), ____],
];
