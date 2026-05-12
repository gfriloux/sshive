// Jeu de données de démonstration — utilisé en mode VITE_MOCK=true

const KEY_001 = 'a1b2c3d4-0001-0001-0001-000000000001';
const KEY_002 = 'a1b2c3d4-0002-0002-0002-000000000002';
const KEY_003 = 'a1b2c3d4-0003-0003-0003-000000000003';
const KEY_004 = 'a1b2c3d4-0004-0004-0004-000000000004';
const SVC_001 = 'b2c3d4e5-0001-0001-0001-000000000001';
const SVC_002 = 'b2c3d4e5-0002-0002-0002-000000000002';
const SVC_003 = 'b2c3d4e5-0003-0003-0003-000000000003';
const SVC_004 = 'b2c3d4e5-0004-0004-0004-000000000004';

export const MOCK_KEYS = [
  {
    id: KEY_001,
    fingerprint: 'SHA256:xK7mP2nQr4vW9yZ1aB3cD5eF6gH8iJ0kLmNoPqRsT',
    key_type: 'ed25519',
    yubikey: false,
    created_at: '2025-01-15',
    comment: 'sshive/github/gfriloux/2025-01-15',
    private_path: '/home/user/.ssh/sshive_github_gfriloux_2025-01-15',
    public_path: '/home/user/.ssh/sshive_github_gfriloux_2025-01-15.pub',
    service_id: SVC_001,
    backup_prompted: false,
    protection: 'protected',
    linked_service_name: 'GitHub gfriloux',
  },
  {
    id: KEY_002,
    fingerprint: 'SHA256:aB3cD5eF6gH8iJ0kLmNoPqRsTuV1wX2yZ3ABCDEF',
    key_type: 'ed25519',
    yubikey: false,
    created_at: '2025-03-22',
    comment: 'sshive/ssh/clochette/2025-03-22',
    private_path: '/home/user/.ssh/sshive_ssh_clochette_2025-03-22',
    public_path: '/home/user/.ssh/sshive_ssh_clochette_2025-03-22.pub',
    service_id: SVC_002,
    backup_prompted: false,
    protection: 'protected',
    linked_service_name: 'guillaume@clochette',
  },
  {
    id: KEY_003,
    fingerprint: 'SHA256:yZ9kL2mN4oP6qR8sTuVwXyZaAbBcCdDeEfFgGhH',
    key_type: 'ed25519',
    yubikey: false,
    created_at: '2025-08-10',
    comment: 'sshive/github/ci-cd/2025-08-10',
    private_path: '/home/user/.ssh/sshive_github_cicd_2025-08-10',
    public_path: '/home/user/.ssh/sshive_github_cicd_2025-08-10.pub',
    service_id: SVC_004,
    backup_prompted: false,
    protection: 'protected',
    linked_service_name: 'GitHub CI/CD',
  },
  {
    id: KEY_004,
    fingerprint: 'SHA256:tU1vW3xY5zA7bC9dEfGhIjKlMnOpQrStUvWxYzA',
    key_type: 'sk-ed25519',
    yubikey: true,
    created_at: '2024-11-05',
    comment: 'sshive/yubikey/5c/2024-11-05',
    private_path: null,
    public_path: '/home/user/.ssh/sshive_yubikey_5c_2024-11-05.pub',
    service_id: null,
    backup_prompted: true,
    protection: 'unknown',
    linked_service_name: null,
  },
];

export const MOCK_SERVICES = [
  {
    id: SVC_001,
    name: 'GitHub gfriloux',
    service_type: 'github',
    params: { url: 'github.com', user: 'gfriloux', port: null, token_ref: 'sshive/github/gfriloux' },
    active_key: KEY_001,
    pending_key: null,
    created_at: '2025-01-15',
    last_rotation: '2025-01-15',
    deploy_mode: 'automatic',
    deployments: [
      { key_id: KEY_001, deployed_at: '2025-01-15', remote_ref: '42857913', last_verified: '2025-03-01' },
    ],
    health_level: 'warning',
    health_reasons: ['Rotation en retard de 15 jours (seuil : 90 jours)'],
    rotation_age_days: 105,
    public_key: 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHxK7mP2nQr4vW9yZ1aB3cD5eF6gH8iJ0kLmNoPqRsT sshive/github/gfriloux/2025-01-15',
  },
  {
    id: SVC_002,
    name: 'guillaume@clochette',
    service_type: 'ssh-generic',
    params: { url: 'clochette.friloux.me', user: 'guillaume', port: 22, token_ref: null },
    active_key: KEY_002,
    pending_key: null,
    created_at: '2025-03-22',
    last_rotation: '2025-03-22',
    deploy_mode: 'automatic',
    deployments: [
      { key_id: KEY_002, deployed_at: '2025-03-22', remote_ref: null, last_verified: '2025-04-10' },
    ],
    health_level: 'ok',
    health_reasons: [],
    rotation_age_days: 50,
    public_key: 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBaB3cD5eF6gH8iJ0kLmNoPqRsTuV1wX2yZ3ABCDEF sshive/ssh/clochette/2025-03-22',
  },
  {
    id: SVC_003,
    name: 'GitLab work',
    service_type: 'gitlab-self-hosted',
    params: { url: 'gitlab.example.corp', user: null, port: null, token_ref: 'sshive/gitlab/work' },
    active_key: null,
    pending_key: null,
    created_at: '2025-09-01',
    last_rotation: null,
    deploy_mode: 'automatic',
    deployments: [],
    health_level: 'critical',
    health_reasons: ['Aucune clef active configurée', 'Token API non configuré'],
    rotation_age_days: null,
    public_key: null,
  },
  {
    id: SVC_004,
    name: 'GitHub CI/CD',
    service_type: 'github',
    params: { url: 'github.com', user: 'gfriloux', port: null, token_ref: null },
    active_key: KEY_003,
    pending_key: null,
    created_at: '2025-08-10',
    last_rotation: '2025-08-10',
    deploy_mode: 'guided',
    deployments: [
      { key_id: KEY_003, deployed_at: '2025-08-10', remote_ref: '98371204', last_verified: '2025-09-01' },
    ],
    health_level: 'warning',
    health_reasons: ['Token API non configuré'],
    rotation_age_days: 18,
    public_key: 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICyZ9kL2mN4oP6qR8sTuVwXyZaAbBcCdDeEfFgGhH sshive/github/ci-cd/2025-08-10',
  },
];

export const MOCK_APP_STATE = {
  services: MOCK_SERVICES,
  keys: MOCK_KEYS,
  health_counts: { ok: 1, warning: 2, critical: 1 },
  gpg_configured: true,
  settings: {
    rotation_warning_days: 90,
    gpg_fingerprint: 'ABCDEF1234567890ABCDEF1234567890ABCDEF12',
    min_passphrase_len: 12,
  },
};

export const MOCK_GPG_KEYS = [
  { fingerprint: 'ABCDEF1234567890ABCDEF1234567890ABCDEF12', name: 'Guillaume Friloux', email: 'guillaume@friloux.me' },
];
