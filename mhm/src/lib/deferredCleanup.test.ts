import { describe, expect, it, vi } from "vitest";
import { createDeferredCleanup } from "./deferredCleanup";

describe("createDeferredCleanup", () => {
    it("runs unlisten once when cleanup happens before registration resolves", async () => {
        const unlisten = vi.fn();
        let resolveRegistration: ((value: () => void) => void) | undefined;
        const registration = new Promise<() => void>((resolve) => {
            resolveRegistration = resolve;
        });

        const cleanup = createDeferredCleanup(registration);
        cleanup();

        resolveRegistration?.(unlisten);
        await Promise.resolve();

        expect(unlisten).toHaveBeenCalledTimes(1);
    });

    it("runs unlisten immediately when cleanup happens after registration resolves", async () => {
        const unlisten = vi.fn();
        const cleanup = createDeferredCleanup(Promise.resolve(unlisten));

        await Promise.resolve();
        cleanup();

        expect(unlisten).toHaveBeenCalledTimes(1);
    });
});
