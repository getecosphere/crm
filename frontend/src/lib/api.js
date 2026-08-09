import { getToken, redirectToLogin } from './session.js';

const AUTH_URL = import.meta.env.PUBLIC_AUTH_URL;
const CRM_URL = import.meta.env.PUBLIC_CRM_URL;
const PARTNERS_URL = import.meta.env.PUBLIC_PARTNERS_URL;
const PRODUCTS_URL = import.meta.env.PUBLIC_PRODUCTS_URL;

export { AUTH_URL, CRM_URL, PARTNERS_URL, PRODUCTS_URL };

async function request(method, url, data, token) {
  const headers = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = `Bearer ${token}`;

  const opts = { method, headers };
  if (data) opts.body = JSON.stringify(data);

  const res = await fetch(url, opts);

  if (res.status === 401) {
    redirectToLogin();
    throw new Error('Unauthorized');
  }

  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message || err.error || 'Request failed');
  }

  return res.json();
}

export function apiGet(url, token) {
  const t = token || getToken();
  return request('GET', url, null, t);
}

export function apiPost(url, data, token) {
  const t = token || getToken();
  return request('POST', url, data, t);
}

export function apiPut(url, data, token) {
  const t = token || getToken();
  return request('PUT', url, data, t);
}

export function apiDelete(url, token) {
  const t = token || getToken();
  return request('DELETE', url, null, t);
}
