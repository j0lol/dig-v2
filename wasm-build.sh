#!/usr/bin/env bash

set -e

HELP_STRING=$(
	cat <<-END
		usage: build_wasm.sh PROJECT_NAME [--release]

		Build script for combining a Macroquad project with wasm-bindgen,
		allowing integration with the greater wasm-ecosystem.

		example: ./build_wasm.sh flappy-bird

		  This'll go through the following steps:

			    1. Build as target 'wasm32-unknown-unknown'.
			    2. Create the directory 'dist' if it doesn't already exist.
			    3. Run wasm-bindgen with output into the 'dist' directory.
		            - If the '--release' flag is provided, the build will be optimized for release.
			    4. Apply patches to the output js file (detailed here: https://github.com/not-fl3/macroquad/issues/212#issuecomment-835276147).
			    5. Generate coresponding 'index.html' file.

			Author: Tom Solberg <me@sbg.dev>
			Edit: Nik codes <nik.code.things@gmail.com>
			Edit: Nobbele <realnobbele@gmail.com>
			Edit: profan <robinhubner@gmail.com>
			Edit: Nik codes <nik.code.things@gmail.com>

			Version: 0.4
	END
)

die() {
	echo >&2 "$HELP_STRING"
	echo >&2
	echo >&2 "Error: $*"
	exit 1
}

# Parse primary commands
while [[ $# -gt 0 ]]; do
	key="$1"
	case $key in
	--release)
		RELEASE=yes
		shift
		;;

	-h | --help)
		echo "$HELP_STRING"
		exit 0
		;;

	*)
		POSITIONAL+=("$1")
		shift
		;;
	esac
done

# Restore positionals
set -- "${POSITIONAL[@]}"
[ $# -ne 1 ] && die "too many arguments provided"

PROJECT_NAME=$1
HASH=$(git rev-parse HEAD)
MANGLED_NAME=$PROJECT_NAME'-'$HASH
HTML=$(
	cat <<-END
		<html lang="en">
		<head>
		    <meta charset="utf-8">
		    <title>${PROJECT_NAME}</title>
		    <style>
		        html,
		        body,
		        canvas {
		            margin: 0px;
		            padding: 0px;
		            width: 100%;
		            height: 100%;
		            overflow: hidden;
		            position: absolute;
		            z-index: 0;
		        }
				.popover {
				    z-index: 10;
				    position: absolute;
				    background: #3e042d;
				    color: white;
				    padding: 1em;
				    margin: 0.2em;
				    font-family: sans-serif;
				    font-size: 8pt;
				}
				.popover > button {
				    font-size: 8pt;
				}
		    </style>
		</head>
		<body style="margin: 0; padding: 0; height: 100vh; width: 100vw; background: black;">
		    <canvas id="glcanvas" tabindex='1' hidden></canvas>
		    <script src="mq_js_bundle.js"></script>
			<script src="sapp_jsutils.js"></script>
			<script src="quad-storage.js"></script>
			
		    <script type="module">
		        import init, { set_wasm } from "./${MANGLED_NAME}.js";
		        async function impl_run() {
		            let wbg = await init();
		            miniquad_add_plugin({
		                register_plugin: (a) => (a.wbg = wbg),
		                on_init: () => set_wasm(wasm_exports),
		                version: "0.0.1",
		                name: "wbg",
		            });
		            load("./${MANGLED_NAME}_bg.wasm");
		        }
		        window.run = function() {
		            document.getElementById("run-container").remove();
		            document.getElementById("glcanvas").removeAttribute("hidden");
		            document.getElementById("glcanvas").focus();
		            impl_run();
		        }
				
                document.addEventListener("contextmenu", function (e){
                    e.preventDefault();
                }, false);
                
                function show_controls() {
                    alert("W/S: left/right.\nSpace: jump.\nSpace x2: Fly for some time\nLeft click: remove block\nRight click: place block\nX: Go to spawn");
                }
                
                function go_away() {
                    document.querySelectorAll(".popover").forEach(el => el.remove());
                }
            </script>
        
            <div class="popover">
                <span> Game made in Macroquad (Rust), compiled to WASM. W.I.P, obviously. </span>
                <br>
                <button onclick="show_controls()">Controls</button>
                <button onclick="go_away()">Go away</button>
            </div>
			
		    <div id="run-container" style="display: flex; justify-content: center; align-items: center; height: 100%; flex-direction: column; z-index: 0; background: black;">
		        <button onclick="run()">Run Game</button>
		    </div>
		</body>
		</html>
	END
)

TARGET_DIR="target/wasm32-unknown-unknown"
# Build
if [ -n "$RELEASE" ]; then
	cargo build --release --target wasm32-unknown-unknown
	TARGET_DIR="$TARGET_DIR/release"
else
	cargo build --target wasm32-unknown-unknown
	TARGET_DIR="$TARGET_DIR/debug"
fi

# Generate bindgen outputs
mkdir -p dist
rm -f dist/$PROJECT_NAME*
wasm-bindgen $TARGET_DIR/"$PROJECT_NAME".wasm --out-dir dist --target web --no-typescript --out-name "$MANGLED_NAME"

# Shim to tie the thing together

if [ "$(uname)" == "Darwin" ]; then
    sed -i '' "s/import \* as __wbg_star0 from 'env';//" dist/"$MANGLED_NAME".js
    sed -i '' "s/let wasm;/let wasm; export const set_wasm = (w) => wasm = w;/" dist/"$MANGLED_NAME".js
    sed -i '' "s/imports\['env'\] = __wbg_star0;/return imports.wbg\;/" dist/"$MANGLED_NAME".js
    sed -i '' "s/const imports = __wbg_get_imports();/return __wbg_get_imports();/" dist/"$MANGLED_NAME".js
else
    sed -i "s/import \* as __wbg_star0 from 'env';//" dist/"$MANGLED_NAME".js
    sed -i "s/let wasm;/let wasm; export const set_wasm = (w) => wasm = w;/" dist/"$MANGLED_NAME".js
    sed -i "s/imports\['env'\] = __wbg_star0;/return imports.wbg\;/" dist/"$MANGLED_NAME".js
    sed -i "s/const imports = __wbg_get_imports();/return __wbg_get_imports();/" dist/"$MANGLED_NAME".js
fi


# Create index from the HTML variable
echo "$HTML" >dist/index.html

pushd dist
wget -O mq_js_bundle.js https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js
wget -O sapp_jsutils.js https://raw.githubusercontent.com/not-fl3/sapp-jsutils/master/js/sapp_jsutils.js
wget -O quad-storage.js https://raw.githubusercontent.com/optozorax/quad-storage/master/js/quad-storage.js
popd