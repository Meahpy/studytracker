import '@fontsource-variable/bricolage-grotesque';
import '@fontsource/figtree/400.css';
import '@fontsource/figtree/500.css';
import '@fontsource/figtree/600.css';

import './styles/tokens.css';
import './styles/base.css';

import { mount } from 'svelte';
import App from './App.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('Elemen #app tidak ditemukan.');

mount(App, { target });
