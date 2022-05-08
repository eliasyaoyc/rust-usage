Deno.core.print("Resolving module base.js\n");

function print(s) {
    Deno.core.print(s);
}

export {print};