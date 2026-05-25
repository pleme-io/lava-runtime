(defcaixa
  :name
  "lava-runtime"
  :kind
  :Biblioteca
  :ecosystem
  :rust-single-crate
  :package
  {:name "lava-runtime"
   :version "0.1.0"
   :description "Unified EmbeddedRuntime trait for in-process IaC DSL evaluation. Wraps lava-eval (tatara-lisp), pangea-ruby-eval (Ruby/magnus), and (future) tatara-script — magma consumes any runtime via one typed surface. One orchestration shape for all DSLs; zero shell-out, zero IPC, zero disk-roundtrip between authoring and apply."
   :license "MIT"
   :repository "https://github.com/pleme-io/lava-runtime"}
  :ci-config
  {:bump {:default-type "patch"}
   :publish {:no-verify true}}
  :workflows
  [:auto-release :pre-merge-gate :security-gate])
