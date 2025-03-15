import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { createRouter, createWebHistory } from 'vue-router';
import App from './App.vue';
import './assets/tailwindcss/main.css';

import Login from './views/login.vue';
import Dashboard from './views/dashboard.vue';
import Users from './views/users.vue';
import Stocks from './views/stocks.vue';
import Supply from './views/supply.vue';
import Transaction from './views/transaction.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/login' },
    { path: '/login', component: Login },
    { 
      path: '/dashboard', 
      component: Dashboard,
      meta: { requiresAuth: true }
    },
    { 
      path: '/users', 
      component: Users,
      meta: { requiresAuth: true, requiresAdmin: true }
    },
    { 
      path: '/stocks', 
      component: Stocks,
      meta: { requiresAuth: true }
    },
    { 
      path: '/supply', 
      component: Supply,
      meta: { requiresAuth: true }
    },
    { 
      path: '/transactions', 
      component: Transaction,
      meta: { requiresAuth: true }
    },
  ]
});

// navigation guard for authentication
router.beforeEach((to, _from, next) => {
  const userJson = localStorage.getItem('user');
  const user = userJson ? JSON.parse(userJson) : null;
  
  if (to.meta.requiresAuth && !user) {
    next('/login');
  } else if (to.meta.requiresAdmin && user?.role !== 'admin') {
    next('/dashboard');
  } else {
    next();
  }
});

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount('#app');