# Envja

Performs environment variable interpolation in a Jinja2-lite syntax.

Contains both the [CLI](cli/) and the [library](lib/) component.

## How to install CLI

### Via `cargo`

```bash
cargo install envja
```

## Simple Examples

### Direct

```bash
envja direct '{% if VAL %}VAL={{VAL}}{% endif %}'
```

Should return empty string because `VAL` env var is missing.

```bash
VAL=hello envja direct '{% if VAL %}VAL={{VAL}}{% endif %}'
```

Should return `VAL=hello`.

### Direct via STDIN

With similar set-up like in [Direct](#direct):

```bash
echo -n '{% if VAL %}VAL={{VAL}}{% endif %}' | envja direct
```

```bash
echo -n '{% if VAL %}VAL={{VAL}}{% endif %}' | VAL=hello envja direct
```

### Via file

With similar set-up like in [Direct](#direct):

```bash
echo -n '{% if VAL %}VAL={{VAL}}{% endif %}' > test.tmpl
envja file test.tmpl
rm test.tmpl
```

```bash
echo -n '{% if VAL %}VAL={{VAL}}{% endif %}' > test.tmpl
VAL=hello envja file test.tmpl
rm test.tmpl
```

## Complex Example

```bash
echo '{% if LINUX_HEADER %}#include <{{LINUX_HEADER}}>{% endif %}
int main() {
    return {{ RET }};
}' > test.tmpl
LINUX_HEADER=unistd.h RET=123 envja file test.tmpl
rm test.tmpl
```

Should print:

```cpp
#include <unistd.h>
int main() {
    return 123;
}
```
