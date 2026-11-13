# Build environment for fframes: `source env.sh` in a bash shell (Git Bash on Windows) before cargo, and
# before running the built binary (it needs the FFmpeg shared libraries at run time too).
#
# fframes links FFmpeg's shared libraries (FFMPEG_DIR) and needs libclang for its bindings (LIBCLANG_PATH).
# Set them in your shell profile, or edit the defaults below. Values already in the environment win.
#
# Windows: a "shared" FFmpeg build (e.g. a gyan.dev or BtbN "*-shared" zip), unpacked anywhere, and LLVM
#   (winget install LLVM.LLVM). FFMPEG_DIR is the folder that holds bin/, include/ and lib/.
#     export FFMPEG_DIR="C:/tools/ffmpeg-shared"        LIBCLANG_PATH="C:/Program Files/LLVM/bin"
# macOS (Homebrew): brew install ffmpeg llvm pkg-config
#     export FFMPEG_DIR="$(brew --prefix ffmpeg)"       LIBCLANG_PATH="$(brew --prefix llvm)/lib"
# Linux (Debian/Ubuntu): apt install libavcodec-dev libavformat-dev libavutil-dev libswscale-dev \
#     libswresample-dev libavfilter-dev libavdevice-dev clang pkg-config
#   (pkg-config finds the system FFmpeg, so FFMPEG_DIR can stay unset; LIBCLANG_PATH only if clang isn't found)

if [ -n "${FFMPEG_DIR:-}" ]; then
  export FFMPEG_DIR
  # The FFmpeg DLLs / libraries must be findable when the binary runs (on Windows, without them on PATH the
  # binary exits with code 127 silently when its output is piped).
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) export PATH="$(cygpath -u "$FFMPEG_DIR")/bin:$PATH" ;;
    Darwin*) export DYLD_LIBRARY_PATH="$FFMPEG_DIR/lib${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" ;;
    *) export LD_LIBRARY_PATH="$FFMPEG_DIR/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ;;
  esac
else
  echo "env.sh: FFMPEG_DIR is not set (fine on Linux with system FFmpeg; on Windows/macOS set it, see the comments)" >&2
fi
if [ -n "${LIBCLANG_PATH:-}" ]; then
  export LIBCLANG_PATH
  case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) export PATH="$(cygpath -u "$LIBCLANG_PATH"):$PATH" ;;
  esac
fi

# The video's command line: R inspect, R strip Idea -n 12, R frame "Idea@end", R render -o out/video.mp4
R() { cargo run --release -q -- "$@"; }
