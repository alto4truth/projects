"""Natural Evolution Strategies (NES) for distributed training."""

import torch
import torch.nn as nn
import numpy as np
from typing import Optional


class NESOptimizer:
    """
    NES - параллелится идеально, без синхронизации градиентов!
    Каждый GPU считает свой fitness, результаты агрегируются.
    """
    
    def __init__(
        self,
        model: nn.Module,
        popsize: int = 256,
        learning_rate: float = 0.01,
        sigma: float = 0.01,
        momentum: float = 0.9,
    ):
        self.model = model
        self.popsize = popsize
        self.lr = learning_rate
        self.sigma = sigma
        self.momentum = momentum
        
        # Центральные параметры
        self.theta = self._get_flat_params()
        self.velocity = torch.zeros_like(self.theta)
        
        # Адаптивная сигма
        self.sigmas = torch.ones(self.theta.shape) * sigma
    
    def _get_flat_params(self) -> torch.Tensor:
        return torch.cat([p.data.flatten() for p in self.model.parameters()])
    
    def _set_flat_params(self, theta: torch.Tensor):
        offset = 0
        for p in self.model.parameters():
            numel = p.numel()
            p.data = theta[offset:offset+numel].reshape(p.shape)
            offset += numel
    
    def sample_population(self) -> list[torch.Tensor]:
        """Семплируем популяцию вокруг текущих параметров."""
        population = []
        for _ in range(self.popsize):
            noise = torch.randn_like(self.theta) * self.sigmas
            # Добавляем шум
            candidate = self.theta + noise
            population.append(candidate)
        return population
    
    def update(self, fitnesses: torch.Tensor):
        """
        Обновляем на основе fitness оценок.
        fitnesses: [popsize] - оценки для каждой особи
        """
        # Ранжирование (rank transformation)
        ranks = torch.argsort(torch.argsort(fitnesses))
        normalized_ranks = (ranks.float() / self.popsize - 0.5) * 2  # [-1, 1]
        
        # Семплируем noise для каждой особи
        noises = []
        for _ in range(self.popsize):
            noise = torch.randn_like(self.theta) * self.sigmas
            noises.append(noise)
        
        # Оцениваем градиент
        gradient = torch.zeros_like(self.theta)
        for i, noise in enumerate(noises):
            gradient += normalized_ranks[i] * noise
        
        gradient /= self.popsize
        gradient *= self.sigmas  # Масштабируем
        
        # Momentum update
        self.velocity = self.momentum * self.velocity + gradient * self.lr
        self.theta += self.velocity
        
        # Адаптивная сигма (увеличиваем для большего explore, если fitness плохой)
        # self.sigmas *= np.exp(0.01 * (fitnesses.mean().item() - 0))
        
        self._set_flat_params(self.theta)
    
    def step(self, fitness_fn):
        """Один шаг NES."""
        # Семплируем популяцию
        population = self.sample_population()
        
        # Оцениваем fitness (распараллелить!)
        fitnesses = []
        for theta in population:
            self._set_flat_params(theta)
            fit = fitness_fn(self.model)
            fitnesses.append(fit)
        
        fitnesses = torch.tensor(fitnesses)
        
        # Восстанавливаем основные параметры
        self._set_flat_params(self.theta)
        
        # Обновляем
        self.update(fitnesses)


class DistributedNES:
    """
    NES легко распараллеливается:
    - Каждый GPU семплит себе особи
    - fitness считается локально
    - агрегация через all_reduce (только скалярные fitness!)
    """
    
    def __init__(self, world_size: int = 8, **nes_kwargs):
        self.world_size = world_size
        self.nes = NESOptimizer(**nes_kwargs)
        self.local_popsize = nes_kwargs.get("popsize", 256) // world_size
    
    def分布式_step(self, fitness_fn, global_fitness_fn):
        """
        1. Каждый GPU считает локальный fitness
        2. All-reduce для получения глобального fitness ranking
        3. Обновление
        """
        import torch.distributed as dist
        
        # Локальное fitness
        local_fitness = fitness_fn(self.nes.theta)
        
        # All-reduce (только tensor ranking, не градиенты!)
        global_fitness_buffer = torch.zeros(self.world_size)
        dist.all_gather(global_fitness_buffer, local_fitness)
        
        # Ranking
        ranks = torch.argsort(torch.argsort(global_fitness_buffer))
        my_rank = ranks[0]  # rank текущего GPU
        
        # Обновляем
        self.nes.update_rank(my_rank)


if __name__ == "__main__":
    model = nn.Linear(10, 10)
    nes = NESOptimizer(model, popsize=16)
    
    def fake_fitness(theta):
        # Глупая функция - работает/не работает
        return -((theta ** 2).sum())
    
    for i in range(10):
        nes.step(fake_fitness)
        if i % 5 == 0:
            print(f"Step {i}, params norm: {nes.theta.norm().item():.4f}")