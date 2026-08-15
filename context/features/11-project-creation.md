# Feature 11: Project Creation

## Goal

Create supported JavaScript projects through official framework scaffolding, then configure, register, and optionally start them in one workflow.

## User Outcome

`nerd create my-app` or desktop wizard creates a ready project available at `https://my-app.test` with chosen Node, package manager, HTTPS, and services.

## In Scope

- Desktop wizard and CLI
- Next.js
- Vite with React, Vue, Svelte, and vanilla
- Nuxt
- Astro
- NestJS
- Location, Node version, package manager, language options where supported
- Optional MySQL, PostgreSQL, Redis
- Optional Git initialization
- Official scaffold CLI execution and live output
- Scaffold package/version and final command preflight
- `nerd.json` generation
- Automatic parked detection or explicit link
- Optional dependency install and first start

## Out Of Scope

- Nerd-maintained framework templates
- Authentication/database application code generation
- Arbitrary remote template execution
- Production deployment

## Workflow

```text
validate -> resolve runtime/tooling -> scaffold temp/target
-> install dependencies -> write nerd.json -> register
-> initialize services -> optionally start -> complete
```

## Rules

- Use official scaffold packages and documented non-interactive flags.
- Show exact framework/package version before execution.
- Existing non-empty destination is rejected unless official workflow safely supports it and user confirms.
- Failure reports retained path and safe cleanup options.
- Wizard indicates when cancellation is safe according to OD-029; it never kills a non-cancellable stage without a recovery plan.
- Never run generated project code elevated.
- Scaffolding and generated dependency scripts require explicit approval.
- Generated `nerd.json` contains no secrets or generated ports.

## Acceptance Criteria

- Every framework option creates and starts a smoke project.
- Wizard and CLI produce equivalent manifest and registration.
- Cancellation leaves no registered half-project.
- Existing files are never silently overwritten.
- Service selections produce valid environment placeholders and healthy instances.
- Created project uses selected Node and package manager.
- Cancellation or scaffold failure never marks generated code trusted automatically.

## Dependencies

- Features 03, 04, 05, 06, 07, and 10
