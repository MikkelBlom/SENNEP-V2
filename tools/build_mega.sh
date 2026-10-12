#!/usr/bin/env bash
# Compile firmware/sennep_mega for the Arduino Mega 2560.
#   tools/build_mega.sh            -> uses arduino-cli if installed (normal case, on the Pi or a PC)
#   tools/build_mega.sh --avr-gcc  -> plain avr-gcc + Ubuntu's arduino-core-avr (used for compile checks in the cloud)
# Output: build/sennep_mega/sennep_mega.ino.hex
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$ROOT/build/sennep_mega"
mkdir -p "$OUT"
python3 "$ROOT/tools/gen_msgs.py" --check

if [[ "${1:-}" != "--avr-gcc" ]] && command -v arduino-cli >/dev/null; then
    arduino-cli compile --fqbn arduino:avr:mega:cpu=atmega2560 \
        --libraries "$ROOT/firmware/libraries" \
        --warnings all --output-dir "$OUT" "$ROOT/firmware/sennep_mega"
    exit 0
fi

CORE=${ARDUINO_AVR_CORE:-/usr/share/arduino/hardware/arduino/avr}
[[ -d "$CORE/cores/arduino" ]] || { echo "no arduino-cli and no AVR core at $CORE (apt install gcc-avr avr-libc arduino-core-avr)"; exit 1; }
OBJ="$OUT/obj"; rm -rf "$OBJ"; mkdir -p "$OBJ"
FLAGS="-mmcu=atmega2560 -DF_CPU=16000000L -DARDUINO=10819 -DARDUINO_AVR_MEGA2560 -DARDUINO_ARCH_AVR -Os -g -flto -ffunction-sections -fdata-sections -Wall -Wextra"
INC="-I$CORE/cores/arduino -I$CORE/variants/mega -I$CORE/libraries/Wire/src -I$CORE/libraries/EEPROM/src -I$ROOT/firmware/libraries/SennepProto/src -I$ROOT/firmware/sennep_mega"
CXX="avr-g++ -std=gnu++11 -fpermissive -fno-exceptions -fno-threadsafe-statics -fno-devirtualize -Wno-error=narrowing"
CC="avr-gcc -std=gnu11"

compile() { # src obj [strict]
    case "$1" in
        *.c) $CC $FLAGS ${3:-} $INC -c "$1" -o "$2" ;;
        *.cpp) $CXX $FLAGS ${3:-} $INC -c "$1" -o "$2" ;;
        *.S) avr-gcc -x assembler-with-cpp $FLAGS $INC -c "$1" -o "$2" ;;
    esac
}
i=0
for f in "$CORE"/cores/arduino/*.{c,cpp,S} "$CORE"/libraries/Wire/src/*.cpp "$CORE"/libraries/Wire/src/utility/*.c; do
    [[ -e "$f" ]] || continue; i=$((i+1)); compile "$f" "$OBJ/core_$i.o" "-w"
done
# Our code: warnings are errors.
for f in "$ROOT"/firmware/libraries/SennepProto/src/*.c "$ROOT"/firmware/sennep_mega/*.cpp; do
    compile "$f" "$OBJ/$(basename "$f").o" "-Werror"
done
{ echo '#include <Arduino.h>'; echo '#line 1 "sennep_mega.ino"'; cat "$ROOT/firmware/sennep_mega/sennep_mega.ino"; } > "$OBJ/sketch.cpp"
compile "$OBJ/sketch.cpp" "$OBJ/sketch.o" "-Werror"
avr-gcc -mmcu=atmega2560 -Os -flto -fuse-linker-plugin -Wl,--gc-sections -o "$OUT/sennep_mega.ino.elf" "$OBJ"/*.o -lm
avr-objcopy -O ihex -R .eeprom "$OUT/sennep_mega.ino.elf" "$OUT/sennep_mega.ino.hex"
avr-size -C --mcu=atmega2560 "$OUT/sennep_mega.ino.elf" | grep -E "Program|Data"
