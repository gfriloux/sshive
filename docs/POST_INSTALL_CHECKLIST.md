# Post-Installation Checklist

After cloning or pulling the latest SSHive code, use this checklist to verify the documentation site is ready to go.

## Prerequisites Check

- [ ] Node.js 22+ installed: `node --version`
- [ ] npm installed: `npm --version`
- [ ] Git installed and configured
- [ ] Access to `/home/kuri/Apps/github/gfriloux/sshive/`

## File Structure Validation

```bash
cd /home/kuri/Apps/github/gfriloux/sshive/docs
bash validate-structure.sh
```

Expected output: All 27 files with ✓

- [ ] All configuration files present
- [ ] All content pages (13 Markdown files)
- [ ] Assets (logo.svg)
- [ ] Styles (custom.css)
- [ ] GitHub Actions workflow
- [ ] Developer documentation

## Dependencies Installation

```bash
cd /home/kuri/Apps/github/gfriloux/sshive/docs
npm install
```

Verify:
- [ ] `node_modules/` directory created
- [ ] `package-lock.json` generated
- [ ] No installation errors
- [ ] astro@^4.0.0 installed
- [ ] @astrojs/starlight@^0.20.0 installed

## Build Test

```bash
npm run build
```

Verify:
- [ ] Build completes successfully (no errors)
- [ ] `dist/` directory created
- [ ] HTML files in `dist/`
- [ ] Assets in `dist/`
- [ ] Build size 2-5 MB (reasonable for static site)

Check build output:
```bash
du -sh dist/
ls -la dist/
```

## Development Server Test

```bash
npm run dev
```

Verify in browser:
- [ ] Server starts at `http://localhost:3000/sshive`
- [ ] Home page loads with hero section
- [ ] Navigation sidebar visible
- [ ] Logo displays correctly
- [ ] Colors applied (SSHive blue #4A80D4)
- [ ] Links clickable

Test navigation:
- [ ] Click each section: Démarrer, Guide, Référence
- [ ] Access all 13 pages without 404 errors
- [ ] Internal links work (e.g., `/installation/`, `/guide/services/`)
- [ ] External links work (GitHub link)

Test features:
- [ ] Search bar functional (Starlight built-in)
- [ ] Dark/light mode toggle works
- [ ] Mobile responsive (resize browser)
- [ ] Code blocks render correctly

Stop server: `Ctrl+C`

## Preview Production Build

```bash
npm run preview
```

Verify:
- [ ] Preview server starts
- [ ] Site works offline (no external dependencies)
- [ ] All assets load from `dist/`
- [ ] No console errors

## Content Verification

No placeholders or TODOs:

```bash
grep -ri "TODO\|PLACEHOLDER\|FIXME" src/content/
```

Expected output: (empty)

- [ ] No "TODO" strings found
- [ ] No "PLACEHOLDER" strings found
- [ ] No "FIXME" strings found

Frontmatter validation:

```bash
find src/content/docs -type f \( -name "*.md" -o -name "*.mdx" \) | while read f; do
  echo "=== $f ==="
  head -5 "$f"
done | grep -E "^===|title:|description:"
```

Verify:
- [ ] All pages have `title` field
- [ ] All pages have `description` field
- [ ] No empty frontmatter sections

## Configuration Verification

Check `astro.config.mjs`:

```bash
grep -E "site:|base:|defaultLocale:" docs/astro.config.mjs
```

Expected:
- [ ] `site: 'https://gfriloux.github.io'`
- [ ] `base: '/sshive'`
- [ ] `defaultLocale: 'root'`
- [ ] Language set to `'fr'` (French)

Check sidebar structure:

```bash
grep -A 30 "sidebar:" docs/astro.config.mjs
```

Verify:
- [ ] 3 main sections (Démarrer, Guide, Référence)
- [ ] Démarrer has 3 items
- [ ] Guide has 5 items
- [ ] Référence has 3 items
- [ ] All links point to existing files

## Styles Verification

Check custom colors in `src/styles/custom.css`:

```bash
cat src/styles/custom.css | grep "sl-color"
```

Verify:
- [ ] `--sl-color-accent: #4A80D4` (blue)
- [ ] `--sl-color-accent-low: #1B2535` (dark blue)
- [ ] `--sl-color-accent-high: #E2E8F4` (light blue)
- [ ] Font stack includes `'JetBrains Mono'`

Visual check:
- [ ] Colors match SSHive brand
- [ ] Monospace font used for code
- [ ] Readable in light and dark modes

## GitHub Actions Setup

Check `.github/workflows/docs.yml`:

```bash
cat .github/workflows/docs.yml | head -15
```

Verify:
- [ ] File exists at correct path
- [ ] Triggers on push to main with docs/* paths
- [ ] Node 22 setup
- [ ] Build step: `npm run build`
- [ ] Deploy step present

Verify permissions:
```bash
grep -A 5 "permissions:" .github/workflows/docs.yml
```

- [ ] `contents: read`
- [ ] `pages: write`
- [ ] `id-token: write`

## Final Checklist

- [ ] All files structure validated
- [ ] Dependencies installed
- [ ] Build successful
- [ ] Dev server works
- [ ] Preview works
- [ ] No broken links
- [ ] No TODOs in content
- [ ] All 13 pages accessible
- [ ] Navigation complete
- [ ] Styles applied
- [ ] Colors correct
- [ ] GitHub Actions configured
- [ ] Ready for deployment

## Deployment Readiness

When all above items are checked:

1. Commit changes:
   ```bash
   git add docs/ .github/workflows/docs.yml DOCS_SETUP_SUMMARY.md
   git commit -m "docs: add Astro + Starlight documentation site"
   ```

2. Push to main:
   ```bash
   git push origin main
   ```

3. Monitor GitHub Actions:
   - Go to `https://github.com/gfriloux/sshive/actions`
   - Watch the `Deploy docs to GitHub Pages` workflow
   - Wait for green checkmark

4. Verify live site:
   - Open `https://gfriloux.github.io/sshive`
   - Verify content matches local preview
   - Test navigation and links

## Troubleshooting

### `npm install` fails
- Check Node version: `node --version` (should be 22+)
- Clear cache: `rm -rf node_modules package-lock.json && npm install`
- Check internet connection

### `npm run build` fails
- Check for console output errors
- Verify all Markdown files have proper frontmatter
- Check file paths in `astro.config.mjs`
- Look for broken import statements in config

### Dev server not accessible
- Check port 3000 is not in use: `lsof -i :3000`
- Verify URL: `http://localhost:3000/sshive` (with `/sshive` base path)
- Check firewall/network settings

### GitHub Actions fails
- Check workflow file syntax: `.github/workflows/docs.yml`
- Verify branch name is `main`
- Check GitHub Pages settings in repo
- Review action logs for specific errors

### Styles not applying
- Clear browser cache: `Ctrl+Shift+Delete`
- Check `src/styles/custom.css` is referenced in config
- Verify CSS syntax (no typos)
- Rebuild: `npm run build`

### Links broken
- Verify paths in `.md` files match config sidebar
- Check relative links use `/` prefix (absolute to base)
- Example: `[Link](/guide/services/)` not `[Link](./services)`

## Support

For issues or questions:
- Review `README.md` in docs folder
- Check `STRUCTURE.md` for detailed documentation
- See `VALIDATION.md` for testing procedures
- Consult [Astro Docs](https://docs.astro.build)
- Consult [Starlight Docs](https://starlight.astro.build)

---

**Status**: Ready to verify after setup
**Last Updated**: 2026-05-11
