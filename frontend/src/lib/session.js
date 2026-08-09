export function getSession() {
  try {
    const raw = localStorage.getItem('crm_session');
    if (!raw) return null;
    const session = JSON.parse(raw);
    if (session.expires_at && session.expires_at < Date.now()) {
      localStorage.removeItem('crm_session');
      return null;
    }
    return session;
  } catch {
    localStorage.removeItem('crm_session');
    return null;
  }
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

export function logout() {
  localStorage.removeItem('crm_session');
  window.location.href = '/auth/login/';
}

export function redirectToLogin() {
  window.location.href = '/auth/login/';
}
