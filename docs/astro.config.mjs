import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://gfriloux.github.io',
  base: '/sshive',
  output: 'static',
  integrations: [
    starlight({
      title: 'SSHive',
      description: 'Gestionnaire de clefs SSH — une clef par service.',
      logo: {
        alt: 'SSHive',
        src: './src/assets/logo.svg',
      },
      social: {
        github: 'https://github.com/gfriloux/sshive',
      },
      sidebar: [
        {
          label: 'Démarrer',
          items: [
            { label: 'Introduction', link: '/' },
            { label: 'Installation', link: '/installation/' },
            { label: 'Démarrage rapide', link: '/quickstart/' },
          ],
        },
        {
          label: 'Guide',
          items: [
            { label: 'Services', link: '/guide/services/' },
            { label: 'Clefs SSH', link: '/guide/keys/' },
            { label: 'Déploiement', link: '/guide/deployment/' },
            { label: 'Santé et diagnostics', link: '/guide/health/' },
            { label: 'Tokens API', link: '/guide/tokens/' },
          ],
        },
        {
          label: 'Référence',
          items: [
            { label: 'Configuration', link: '/reference/configuration/' },
            { label: 'Sécurité', link: '/reference/security/' },
            { label: 'Changelog', link: '/reference/changelog/' },
          ],
        },
      ],
      customCss: ['./src/styles/custom.css'],
      defaultLocale: 'root',
      locales: {
        root: {
          label: 'Français',
          lang: 'fr',
        },
      },
    }),
  ],
});
