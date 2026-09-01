// @ts-check
import eslint from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";

export default tseslint.config(
  {
    ignores: ["**/dist/**", "**/coverage/**", "**/node_modules/**", "backend/**"],
  },
  eslint.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    files: ["**/*.{ts,tsx}"],
    rules: {
      "@typescript-eslint/consistent-type-imports": [
        "error",
        { prefer: "type-imports", fixStyle: "inline-type-imports" },
      ],
    },
  },
  {
    files: ["apps/tenant-console/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            // ADR 0001: console surfaces must not import each other.
            {
              group: ["@araf/operator-console", "**/apps/operator-console/**"],
              message: "Tenant Console must not import the Operator Console surface.",
            },
            // ADR 0004: Cloudscape is confined to packages/ui.
            {
              group: ["@cloudscape-design/**"],
              message: "Cloudscape imports are confined to packages/ui (ADR 0004).",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["apps/operator-console/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            // ADR 0001: console surfaces must not import each other.
            {
              group: ["@araf/tenant-console", "**/apps/tenant-console/**"],
              message: "Operator Console must not import the Tenant Console surface.",
            },
            // ADR 0004: Cloudscape is confined to packages/ui.
            {
              group: ["@cloudscape-design/**"],
              message: "Cloudscape imports are confined to packages/ui (ADR 0004).",
            },
          ],
        },
      ],
    },
  },
  {
    // ADR 0004: third-party component primitives are confined to packages/ui.
    files: ["packages/**/*.{ts,tsx}"],
    ignores: ["packages/ui/**/*.{ts,tsx}"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["@cloudscape-design/**"],
              message: "Cloudscape imports are confined to packages/ui (ADR 0004).",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["apps/**/*.{ts,tsx}", "packages/ui/**/*.{ts,tsx}"],
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": ["warn", { allowConstantExport: true }],
    },
  },
  {
    // ADR 0004: third-party component primitives are confined to packages/ui.
    files: ["packages/ui/**/*.{ts,tsx}"],
    rules: {},
  },
  {
    files: ["**/*.test.{ts,tsx}", "**/src/test/**"],
    languageOptions: {
      globals: globals.node,
    },
  },
  {
    files: ["**/*.{mjs,cjs,js}"],
    ...tseslint.configs.disableTypeChecked,
  },
  {
    files: ["**/*.config.{ts,mjs}", "eslint.config.mjs"],
    languageOptions: {
      globals: globals.node,
    },
  },
  {
    // Playwright tests and config are Node-only and do not need type-aware linting.
    files: ["e2e/**/*.ts", "playwright.config.ts"],
    languageOptions: {
      globals: globals.node,
    },
    ...tseslint.configs.disableTypeChecked,
  },
);
