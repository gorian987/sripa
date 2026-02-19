import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import neverthrow from 'eslint-plugin-neverthrow';

export default tseslint.config(
    js.configs.recommended,
    ...tseslint.configs.recommended,
    {
        files: ['**/*.ts', '**/*.svelte'],
        plugins: {
            neverthrow,
        },
        languageOptions: {
            parserOptions: {
                project: ['./tsconfig.json', './tsconfig.node.json'],
                tsconfigRootDir: import.meta.dirname,
            },
        },
        rules: {
            'neverthrow/must-use-result': 'error',
        },
    }
);