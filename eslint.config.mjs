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
      // ADR 0001: console surfaces must not import each other.
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["@araf/operator-console", "**/apps/operator-console/**"],
              message: "Tenant Console must not import the Operator Console surface.",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["apps/operator-console/**/*.{ts,tsx}"],
    rules: {
      // ADR 0001: console surfaces must not import each other.
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: ["@araf/tenant-console", "**/apps/tenant-console/**"],
              message: "Operator Console must not import the Tenant Console surface.",
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
    // No underlying enterprise component library is imported yet (M1); this
    // rule is the enforcement point once one is adopted.
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
);
