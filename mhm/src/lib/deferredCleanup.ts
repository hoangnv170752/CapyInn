export function createDeferredCleanup(registration: Promise<() => void>): () => void {
    let active = true;
    let unlisten: (() => void) | null = null;

    void registration.then((cleanup) => {
        if (!active) {
            cleanup();
            return;
        }
        unlisten = cleanup;
    });

    return () => {
        active = false;
        if (unlisten) {
            const cleanup = unlisten;
            unlisten = null;
            cleanup();
        }
    };
}
