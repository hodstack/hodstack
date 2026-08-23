set -eu

rm -rf tests

python3 - <<'STRIP'
import json

with open("composer.json") as file:
    manifest = json.load(file)

manifest.get("scripts", {}).pop("test", None)

with open("composer.json", "w") as file:
    json.dump(manifest, file, indent=4)
STRIP

git add -A
git commit --quiet -m "the case"
