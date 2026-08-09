const SESSION_KEY = 'crm_session';

export function getSession() {
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    if (!raw) return null;
    const session = JSON.parse(raw);
    if (session.expires_at && session.expires_at < Date.now()) {
      localStorage.removeItem(SESSION_KEY);
      return null;
    }
    return session;
  } catch {
    localStorage.removeItem(SESSION_KEY);
    return null;
  }
}

export function saveSession({ token, user, expires_in }) {
  const session = {
    token,
    user,
    expires_at: Date.now() + ((expires_in || 86400) * 1000),
  };
  localStorage.setItem(SESSION_KEY, JSON.stringify(session));
  return session;
}

export function clearSession() {
  localStorage.removeItem(SESSION_KEY);
}

export function isLoggedIn() {
  return !!getSession();
}

export function getUser() {
  const session = getSession();
  return session ? session.user : null;
}

export function getToken() {
  const session = getSession();
  return session ? session.token : null;
}

export function redirectToLogin() {
  window.location.href = '/auth/login/';
}

export function logout() {
  clearSession();
  window.location.href = '/auth/login/';
}
