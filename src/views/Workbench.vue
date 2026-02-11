<template>
  <div class="page-container" style="padding: 24px 24px 56px 24px;">
    <h1>工作台 (Workbench)</h1>
    <el-row :gutter="20">
      <el-col :span="8" v-for="i in 3" :key="i">
        <el-card shadow="hover">
          <template #header>任务 {{ i }}</template>
          当前进行中的任务...
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>
