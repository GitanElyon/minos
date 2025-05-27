cd rust
maturin build --release
cd ..
pip install .\rust\target\wheels\tetris_bot_rust-0.1.0-cp313-cp313-win_amd64.whl --force-reinstall
echo Build complete!