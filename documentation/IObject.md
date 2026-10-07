# IObject

## Introduction

This file serves as a template for a potential cpp, rust and python implementation of how we store map information.$

## Code

### C++

```C++
/*
** EPITECH PROJECT, 2026
** Zapfrites
** File description:
** IObject.hpp
*/

#pragma once

#include <vector>
#include <memory>

enum EObjectType : unsigned char {
  AI,
  FOOD,
  LINEMATE,
  DERAUMERE,
  SIBUR,
  MENDIANE,
  PHIRAS,
  THYSTAME,
  EGG
};

class IObject {
public:
  virtual ~IObject() = default;

private:
  EObjectType type;
  std::vector<int> position;
};

class CTile {
public:
  CTile();
  ~CTile() = default;
  std::vector<std::unique_ptr<IObject*>> getObjectList();

private:
  std::vector<std::unique_ptr<IObject*>> objects;
};

struct STile {
    CTile tile;

    std::shared_ptr<STile> up;
    std::shared_ptr<STile> down;
    std::shared_ptr<STile> left;
    std::shared_ptr<STile> right;
};
```

### Rust

```rust
/*
** EPITECH PROJECT, 2026
** Zapfrites
** File description:
** IObject.rs
*/

use std::rc::{Rc, Weak};
use std::vec::Vec;

pub enum EObjectType : unsigned char {
  AI,
  FOOD,
  LINEMATE,
  DERAUMERE,
  SIBUR,
  MENDIANE,
  PHIRAS,
  THYSTAME,
  EGG
};

pub struct Object {
  pub Type : EObjectType,
  pub position : Vec<int>,
};

pub struct CTile {
  pub objects : Vec<Rc<Object>>,
};

pub struct STile {
    pub tile : CTile;

    pub up : Rc<STile>;
    pub down : Rc<STile>;
    pub left : Rc<STile>;
    pub right : Rc<STile>;
};
```

### Python

```python
##
## EPITECH PROJECT, 2026
## Zapfrites
## File description:
## IObject.py
##

from enum import Enum

class EObjectType(Enum):
  AI = 0
  FOOD = 1
  LINEMATE = 2
  DERAUMERE = 3
  SIBUR = 4
  MENDIANE = 5
  PHIRAS = 6
  THYSTAME = 7
  EGG = 8

```
