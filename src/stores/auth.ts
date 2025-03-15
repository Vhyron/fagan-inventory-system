import { defineStore } from 'pinia';
import { ref } from 'vue';

interface User {
  id: string;
  username: string;
  role: string;
}

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null);
  
  // Initialize user from localStorage if available
  const storedUser = localStorage.getItem('user');
  if (storedUser) {
    user.value = JSON.parse(storedUser);
  }
  
  function setUser(userData: User | null) {
    user.value = userData;
    if (userData) {
      localStorage.setItem('user', JSON.stringify(userData));
    } else {
      localStorage.removeItem('user');
    }
  }
  
  function logout() {
    user.value = null;
    localStorage.removeItem('user');
  }
  
  function isAdmin() {
    return user.value?.role === 'admin';
  }
  
  function isLoggedIn() {
    return !!user.value;
  }
  
  return {
    user,
    setUser,
    logout,
    isAdmin,
    isLoggedIn
  };
});