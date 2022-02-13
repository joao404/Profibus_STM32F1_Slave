/*********************************************************************
 * Observer
 *
 * Copyright (C) 2022 Marcel Maage
 * 
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * LICENSE file for more details.
 */

#pragma once

#include <Arduino.h>
#include <vector>

class Observable; 

class Observer 
{ 
public:
    virtual ~Observer() = default;
    virtual void update(Observable& observable, void* data) = 0;
};

class Observable 
{ 
public: 
     virtual ~Observable() = default;
     void attach(Observer& observer) { m_observer.push_back(&observer); }
     void detach(Observer& observer)
     {
         m_observer.erase(std::remove(m_observer.begin(), m_observer.end(), &observer));
     }
     void notify(void* data)
     {
         for (auto* observer : m_observer) {
             observer->update(*this, data);
         }
     }
private:
     std::vector<Observer*> m_observer; 
};