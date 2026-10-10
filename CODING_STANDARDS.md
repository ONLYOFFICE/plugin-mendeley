## Always Apply Clean Code Principles

Apply clean code principles in every piece of code you write.

- Simplicity (KISS): Keep logic straightforward and avoid unnecessary complexity. Choose the easiest solution to understand and maintain.
- Readability: Use clear, descriptive names for variables and functions, and keep formatting and indentation consistent.
- Single Responsibility (SRP): Each function, class, or module should have one responsibility and one reason to change.
- Don't Repeat Yourself (DRY): Avoid duplication by extracting repeated logic into reusable functions or modules.
- Small Functions: Keep functions short and focused on a single task to improve clarity and testability.
- Minimal Side Effects: Avoid changing state outside a function's scope unless that behavior is intentional.
- YAGNI: Do not add functionality unless it is immediately needed. Avoid over-engineering.
- Consistency: Follow the codebase's existing conventions and style so the project stays easy to work on.

## I prefer stupid simple code instead of smart one

## No need to create fallback and backward compatibility unless user asking to do so

## Gunakan Redux sebagai State Management, instead hardcode useState

## dont use dash for comment

dont write comment use dash like this
`# ── Preprocessing ──────────────────────────────────────────────────────────────`

better write like this
`#PreProcessing`

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Response Time must be <1000ms

## Separate each function into each file berdasarkan tugasnya masing masing, like logic, hooks, constant, helper, type, etc

## wajib menambahkan/menyertakan logger/logging setiap event/state, etc untuk kemudahan observability, tampilkan semua level logging (debug, info, error, success, etc), format log berbentuk json yang mudah untuk dipantau

## setiap commit, sertakan file/bikin file untuk mencatat perubahan apa pada commit itu

## Untuk migrasi, jangan bikin manual, cukup gunakan bunx prisma
