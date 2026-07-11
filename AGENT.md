# Agent Rules

## Git & PR — WAJIB

### Tidak ada atribusi AI
- **DILARANG** mencantumkan "Generated with Claude Code", "Co-authored-by", "@claude", atau atribusi AI apapun di commit message, PR body, atau kode.
- Author commit = Aji Anaz `<aji.anaz@gmail.com>`.
- Co-author trailer = **selalu kosong**.
- Tidak boleh ada footer signature agent di output.

### Commit message
- Konvensional: `fix:`, `feat:`, `chore:`, `refactor:`, `docs:`, `checkpoint:`
- Bahasa Inggris untuk commit, PR boleh Indonesia.

### Branch
- Naming: `fix/<desc>`, `feat/<desc>`, `checkpoint/<desc>`
- Base PR default: `develop`

## Workflow — WAJIB
- **JANGAN commit/push/PR sebelum user confirm perubahan working di app.**
- Test lokal + user verify dulu, baru commit.
- Kalau gagal/berubah, jangan langsung commit ulang.
