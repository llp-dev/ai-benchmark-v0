FROM alpine:3.20

ENV LANG=C.UTF-8 \
    LC_ALL=C.UTF-8 \
    HOME=/home/runner

# Toolchain note:
#   - `rust` package on Alpine ships rustc (and stdlib).
#   - `cargo` is intentionally NOT installed: the benchmark forbids cargo usage.
#   - `rust-analyzer` provides LSP support for opencode.
RUN apk add --no-cache \
        # core shell + network
        bash \
        ca-certificates \
        curl \
        wget \
        git \
        openssh-client \
        unzip \
        tar \
        xz \
        gzip \
        # GNU userland (default Alpine ships busybox for these — agents expect GNU semantics)
        coreutils \
        findutils \
        grep \
        sed \
        gawk \
        diffutils \
        patch \
        less \
        # search / explore
        ripgrep \
        fd \
        tree \
        file \
        jq \
        yq \
        # build driver
        make \
        # C toolchain (used by the validation/ harness; not part of the model task)
        gcc \
        binutils \
        musl-dev \
        libbsd-dev \
        # Rust toolchain (rustc only, no cargo)
        rust \
        rust-analyzer \
        # debuggers / runtime analysis
        lldb \
        gdb \
        strace \
        # extra runtimes opencode or generated test scripts may shell out to
        nodejs \
        npm \
        python3 \
        py3-pip \
        # bun (opencode runtime) needs these on Alpine
        libstdc++ \
        libgcc \
        gcompat

RUN adduser -D -s /bin/bash runner

USER runner
WORKDIR /home/runner

# Install opencode via the official script (pulls a bun-based binary)
RUN curl -fsSL https://opencode.ai/install | bash

ENV PATH="/home/runner/.opencode/bin:${PATH}"

# Bake the one-time SQLite migration into the image. Fresh ephemeral containers
# would otherwise re-run this migration on every `docker run --rm`, eating
# 60–120 s before the agent can begin work. We invoke a quick `opencode run`
# with a fake provider so the migration completes and gets persisted into the
# image layer at /home/runner/.local/share/opencode/.
RUN OPENROUTER_API_KEY=fake-key-for-migration timeout 180 \
        opencode run --format json -m "openrouter/anthropic/claude-3.5-sonnet" "noop" \
        > /dev/null 2>&1 || true

WORKDIR /work

CMD ["opencode", "--help"]
