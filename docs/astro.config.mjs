import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://gfriloux.github.io',
  base: '/sshive',
  output: 'static',
  integrations: [
    starlight({
      title: 'SSHive',
      description: 'SSH key manager — one key per service.',
      logo: {
        alt: 'SSHive',
        src: './src/assets/logo.svg',
      },
      social: {
        github: 'https://github.com/gfriloux/sshive',
      },
      defaultLocale: 'root',
      locales: {
        root: { label: 'English', lang: 'en' },
        fr:   { label: 'Français', lang: 'fr' },
      },
      sidebar: [
        {
          label: 'Get Started',
          translations: { fr: 'Démarrer' },
          items: [
            { label: 'Introduction',   translations: { fr: 'Introduction' },     link: '/' },
            { label: 'Installation',   translations: { fr: 'Installation' },     link: '/installation/' },
            { label: 'Quick Start',    translations: { fr: 'Démarrage rapide' }, link: '/quickstart/' },
          ],
        },
        {
          label: 'Guide',
          translations: { fr: 'Guide' },
          items: [
            { label: 'Services',           translations: { fr: 'Services' },             link: '/guide/services/' },
            { label: 'SSH Keys',           translations: { fr: 'Clefs SSH' },            link: '/guide/keys/' },
            { label: 'Deployment',         translations: { fr: 'Déploiement' },          link: '/guide/deployment/' },
            { label: 'Health & Diagnosis', translations: { fr: 'Santé et diagnostics' }, link: '/guide/health/' },
            { label: 'API Tokens',         translations: { fr: 'Tokens API' },           link: '/guide/tokens/' },
          ],
        },
        {
          label: 'Reference',
          translations: { fr: 'Référence' },
          items: [
            { label: 'Configuration', translations: { fr: 'Configuration' }, link: '/reference/configuration/' },
            { label: 'Security',      translations: { fr: 'Sécurité' },      link: '/reference/security/' },
            { label: 'Changelog',     translations: { fr: 'Changelog' },     link: '/reference/changelog/' },
          ],
        },
      ],
      customCss: ['./src/styles/custom.css'],
    }),
    // Neutralise @astrojs/sitemap (bug avec base path dans Starlight 0.20)
    {
      name: 'disable-sitemap',
      hooks: {
        'astro:config:setup': ({ config }) => {
          config.integrations = config.integrations.filter(
            (i) => i.name !== '@astrojs/sitemap'
          );
        },
      },
    },
  ],
});
