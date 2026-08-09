import { getToken, clearSession, redirectToLogin } from './session.js';

export const CRM_API = (import.meta.env.PUBLIC_CRM_URL || '/api').replace(/\/$/, '');
export const AUTH_API = (import.meta.env.PUBLIC_AUTH_URL || '/api').replace(/\/$/, '');

async function request(method, url, data, token) {
  const headers = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const opts = { method, headers };
  if (data !== undefined) opts.body = JSON.stringify(data);

  const res = await fetch(url, opts);

  if (res.status === 401) {
    clearSession();
    redirectToLogin();
    throw new Error('Session expired — please sign in again');
  }

  if (!res.ok) {
    let message = 'Request failed';
    try {
      const err = await res.json();
      message = err.error || err.message || message;
    } catch {
      message = res.statusText || message;
    }
    throw new Error(message);
  }
  return res.json();
}

export function crmGet(path, token) {
  const t = token || getToken();
  return request('GET', `${CRM_API}${path}`, undefined, t);
}
export function crmPost(path, data, token) {
  const t = token || getToken();
  return request('POST', `${CRM_API}${path}`, data || {}, t);
}
export function crmPut(path, data, token) {
  const t = token || getToken();
  return request('PUT', `${CRM_API}${path}`, data || {}, t);
}

export function authRequest(method, path, data) {
  return request(method, `${AUTH_API}${path}`, data);
}
