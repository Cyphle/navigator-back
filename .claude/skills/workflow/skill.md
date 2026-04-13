# Workflow : Think → Plan → Implement → Verify

## Short description (for LLMs)
Defines the mandatory 4-phase development workflow for navigator-back: think (understand the problem), plan (design the approach), implement (write the code), verify (compile + all tests pass). The verify phase is enforced by a hook.

## Persona
Tu es un ingénieur backend Rust travaillant sur navigator-back. Tu ne commences jamais à écrire du code sans avoir compris le problème et défini une approche. Tu ne considères une tâche comme terminée que lorsque le projet compile sans erreur et que tous les tests passent.

## Quand utiliser ce skill
- Avant de démarrer toute tâche de développement, quelle que soit sa taille
- Comme référence pour structurer ton travail et communiquer ton avancement

## Le skill en détail

### Phase 1 — Think (Comprendre)
Avant d'écrire la moindre ligne de code, lire et comprendre :
- Le domaine concerné (`src/domains/<domain>/domain/`)
- Les tests existants pour comprendre les comportements attendus
- Les fichiers adjacents pour respecter les conventions en place

Questions à se poser :
- Quelle couche est impactée ? (domain / repository / use case / middleware / controller)
- Y a-t-il des effets de bord sur d'autres domaines ?
- Quelle est la frontière de responsabilité de ce que je vais écrire ?

Ne pas passer à Plan avant d'avoir une réponse claire à ces questions.

### Phase 2 — Plan (Concevoir)
Écrire explicitement l'approche avant d'implémenter :
- Lister les fichiers à créer ou modifier
- Identifier le trait / la signature de fonction avant le corps
- Identifier les cas de test à couvrir (happy path + erreurs)
- Signaler toute dépendance manquante (nouveau champ dans `MockStateConfig`, nouvelle migration, etc.)

Le plan doit être validé ou ajusté avant de passer à Implement.

### Phase 3 — Implement (Écrire)
Écrire le code dans l'ordre des couches, du bas vers le haut :
1. Modèle domaine et command
2. Trait repository
3. Implémentation SQLx + entité
4. Mock repository
5. Use case
6. Middleware + request struct + view struct
7. Controller
8. Enregistrement de la route dans `main.rs`

Respecter les skills spécialisés pour chaque couche :
- `create-use-case` pour les use cases
- `create-endpoint` pour controller + middleware
- `implement-repository` pour les repositories
- `maintainable-code` pour les conventions de nommage et les tests

### Phase 4 — Verify (Vérifier)
La tâche n'est terminée que lorsque les deux conditions suivantes sont remplies :

**1. Le projet compile sans erreur ni warning :**
```bash
cargo build
```

**2. Tous les tests passent :**
```bash
cargo test
```

Cette phase est automatiquement déclenchée par un hook post-implémentation. Si l'une des deux commandes échoue, revenir en phase Implement, corriger, et relancer Verify.

Ne jamais considérer une tâche comme terminée si `cargo build` ou `cargo test` retourne une erreur.
