/**
 * Custom render helper that wraps components with necessary providers.
 */
import { render, type RenderOptions } from "@testing-library/react";
import type { ReactElement } from "react";

/**
 * Custom render function — wraps with any providers if needed.
 * Currently MHM uses zustand (global stores) so no provider needed.
 */
function customRender(ui: ReactElement, options?: Omit<RenderOptions, "wrapper">) {
    return render(ui, { ...options });
}

// Re-export everything from RTL
export * from "@testing-library/react";
export { customRender as render };
