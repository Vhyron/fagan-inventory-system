<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import AppLayout from '../components/app_layout.vue';
import { useAuthStore } from '../stores/auth';

interface User {
  id: string;
  username: string;
  role: string;
}

const authStore = useAuthStore();
const users = ref<User[]>([]);
const loading = ref(true);
const error = ref('');

// new user form
const showNewUserForm = ref(false);
const newUsername = ref('');
const newPassword = ref('');
const formError = ref('');
const formSuccess = ref('');

// fetch all users
async function loadUsers() {
  try {
    loading.value = true;
    users.value = await invoke('get_users');
  } catch (err) {
    console.error('Error loading users:', err);
    error.value = 'Failed to load users';
  } finally {
    loading.value = false;
  }
}

// create new secretary account
async function createSecretary() {
  if (!newUsername.value || !newPassword.value) {
    formError.value = 'Username and password are required';
    return;
  }
  
  try {
    const response = await invoke('create_secretary', {
      request: {
        username: newUsername.value,
        password: newPassword.value
      },
      adminUsername: authStore.user?.username
    });
    
    if (response.success) {
      formSuccess.value = 'Secretary account created successfully';
      formError.value = '';
      newUsername.value = '';
      newPassword.value = '';
      loadUsers(); // refresh
    } else {
      formError.value = response.message;
      formSuccess.value = '';
    }
  } catch (err) {
    console.error('Error creating secretary:', err);
    formError.value = 'Failed to create secretary account';
    formSuccess.value = '';
  }
}

// deactivate a secretary account
async function deactivateSecretary(userId: string) {
  if (!confirm('Are you sure you want to deactivate this secretary account?')) {
    return;
  }
  
  try {
    const response = await invoke('deactivate_secretary', {
      userId,
      adminUsername: authStore.user?.username
    });
    
    if (response.success) {
      loadUsers(); // refresh
    } else {
      error.value = response.message;
    }
  } catch (err) {
    console.error('Error deactivating secretary:', err);
    error.value = 'Failed to deactivate secretary account';
  }
}

// load users on component mount
onMounted(() => {
  loadUsers();
});
</script>

<template>
  <AppLayout>
    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-semibold">User Management</h1>
        <button 
          @click="showNewUserForm = !showNewUserForm" 
          class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
        >
          {{ showNewUserForm ? 'Cancel' : 'Add Secretary' }}
        </button>
      </div>
      
      <!-- add secretary form -->
      <div v-if="showNewUserForm" class="mb-6 p-4 border border-gray-200 rounded">
        <h2 class="text-lg font-medium mb-4">Create New Secretary Account</h2>
        
        <div v-if="formSuccess" class="mb-4 p-3 bg-green-100 text-green-700 rounded">
          {{ formSuccess }}
        </div>
        
        <div v-if="formError" class="mb-4 p-3 bg-red-100 text-red-700 rounded">
          {{ formError }}
        </div>
        
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700">Username</label>
            <input 
              v-model="newUsername" 
              type="text" 
              class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md"
            >
          </div>
          
          <div>
            <label class="block text-sm font-medium text-gray-700">Password</label>
            <input 
              v-model="newPassword" 
              type="password" 
              class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md"
            >
          </div>
          
          <button 
            @click="createSecretary" 
            class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Create Account
          </button>
        </div>
      </div>
      
      <!-- user List -->
      <div v-if="loading" class="text-center py-4">
        Loading users...
      </div>
      
      <div v-else-if="error" class="text-center py-4 text-red-600">
        {{ error }}
      </div>
      
      <div v-else class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Username</th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Role</th>
              <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            <tr v-for="user in users" :key="user.id">
              <td class="px-6 py-4 whitespace-nowrap">{{ user.username }}</td>
              <td class="px-6 py-4 whitespace-nowrap capitalize">{{ user.role }}</td>
              <td class="px-6 py-4 whitespace-nowrap text-right">
                <button 
                  v-if="user.role === 'secretary'"
                  @click="deactivateSecretary(user.id)" 
                  class="text-red-600 hover:text-red-900"
                >
                  Deactivate
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </AppLayout>
</template>