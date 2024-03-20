# Текстурированная стена

В следующем разделе мы собираемся отказаться от чайника и нарисовать стену.

## Стена

Поскольку это довольно простая форма, мы можем построить ее сами:

```rust
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

implement_vertex!(Vertex, position, normal);

let shape = glium::vertex::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0] },
        Vertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0] },
        Vertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0] },
        Vertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0] },
    ]).unwrap();
```

У нас есть только четыре вершины. Причина в том, что мы собираемся использовать треугольную полосу. С полосой треугольника графический процессор нарисует один треугольник с вершинами 0, 1 и 2, а другой треугольник с вершинами 1, 2 и 3. Полоса треугольника очень полезна при рисовании прямоугольников или прямоугольных фигур.

```rust
target.draw(&shape, glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &program,
            &uniform! { model: model, view: view, perspective: perspective, u_light: light },
            &params).unwrap();
```

Остальная часть кода в основном такая же, как и раньше. Вы должны знать, как нарисовать что-то!

![Как это выглядит после шага 1](../tuto-14-step1.png)

## Применение текстуры

Чтобы применить текстуру, мы делаем то же самое, что и несколько разделов ранее:

 - Загружаем текстуру при инициализации.
 - Мы добавляем атрибут `tex_coords` к вершинам.
 - Мы передаем текстуру как униформу.
 - Мы получаем окружающие и диффузные цвета из текстуры.

Загрузка текстуры выполняется так, как мы это уже делали ранее:

```rust
let image = image::load(Cursor::new(&include_bytes!("../book/tuto-14-diffuse.jpg")[..]),
                        image::JPEG).unwrap().to_rgba();
let image_dimensions = image.dimensions();
let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
let diffuse_texture = glium::texture::SrgbTexture2d::new(&display, image).unwrap();
```

Добавление текстурных координат также очень просто:

```rust
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, normal, tex_coords);
```

Передача текстуры включает добавление новой униформы в наш фрагментный шейдер:

```glsl
uniform sampler2D diffuse_tex;
```

И передать его при рисовании:

```rust
target.draw(&shape, glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &program,
            &uniform! { model: model, view: view, perspective: perspective,
                        u_light: light, diffuse_tex: &diffuse_texture },
            &params).unwrap();
```

И затем в фрагментном шейдере мы вместо этого загружаем диффузный и окружающий цвета из текстуры.

Мы просто заменим это:

```glsl
const vec3 ambient_color = vec3(0.2, 0.0, 0.0);
const vec3 diffuse_color = vec3(0.6, 0.0, 0.0);
```

На это:

```glsl
vec3 diffuse_color = texture(diffuse_tex, v_tex_coords).rgb;
vec3 ambient_color = diffuse_color * 0.1;
```

И мы должны получить текстурированную стену!

![Текстурированная стена](../tuto-14-step2.png)

## Карты нормалей

Однако результат не велик. Вы можете ясно видеть, что это просто прямоугольник с нарисованной на нем стеной, а не настоящая стена.

Существует методика, которая может значительно улучшить качество рендеринга: нормальное отображение.

Проблема нашего текущего рендеринга в том, что свет не проникает между камнями. Если бы каждый отдельный камень рисовался один за другим, рендеринг был бы намного лучше благодаря освещению.

Нормальное отображение состоит в корректировке расчета освещения нашего прямоугольника, чтобы сделать так, как будто там были отдельные камни. Это делается путем предоставления нормального *для каждого фрагмента*. Если вы помните, нормаль - это вектор, перпендикулярный поверхности в определенном месте. Используя более мелкозернистые нормали, мы также можем заставить пользователя поверить, что сама поверхность мелкозернистая.

Вот что такое *карта нормалей*:

![Карта нормалей](../tuto-14-normal.png)

Как вы можете видеть, есть много общего с обычной текстурой. Каждый пиксель карты нормалей представляет значение нормали в местоположении этого пикселя. Вместо хранения цветов мы храним произвольные значения, которые представляют нормаль. Например, карты нормалей часто синие, потому что синий - это значение `(0.0, 0.0, 1.0)`, которое представляет собой вектор, указывающий на внешнюю сторону.

Начнем с самого начала. Загружаем карту нормалей в текстуру:

```rust
let image = image::load(Cursor::new(&include_bytes!("../book/tuto-14-normal.png")[..]),
                        image::PNG).unwrap().to_rgba();
let image_dimensions = image.dimensions();
let image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
let normal_map = glium::texture::Texture2d::new(&display, image).unwrap();
```

И мы добавляем новую униформу в наш фрагментный шейдер:

```glsl
uniform sampler2D normal_tex;
```

Теперь вместо использования значения `v_normal`, полученного из нашего вершинного шейдера, мы собираемся загрузить нормаль с карты нормалей, аналогично тому, как мы загружаем диффузный цвет из диффузной текстуры.

```glsl
vec3 normal_map = texture(normal_tex, v_tex_coords).rgb;
```

Однако есть проблема. Значение, хранящееся в карте нормалей, содержит векторы нормалей относительно поверхности объекта. Но во время наших расчетов мы находимся в координатах сцены относительно камеры. Нам нужно умножить значение, которое мы загружаем из карты нормалей, на матрицу, чтобы получить полезные значения. Эта матрица называется матрицей **TBN** (для *касательной бинормальной нормали*).

В прошлом некоторые из вычислений, требуемых для этой матрицы, были предварительно вычислены и переданы как атрибуты. Но вычисление этого на лету действительно практично. Вот функция из [http://www.thetenthplanet.de/archives/1180](http://www.thetenthplanet.de/archives/1180), которая вычисляет ее:

```glsl
mat3 cotangent_frame(vec3 normal, vec3 pos, vec2 uv) {
    vec3 dp1 = dFdx(pos);
    vec3 dp2 = dFdy(pos);
    vec2 duv1 = dFdx(uv);
    vec2 duv2 = dFdy(uv);

    vec3 dp2perp = cross(dp2, normal);
    vec3 dp1perp = cross(normal, dp1);
    vec3 T = dp2perp * duv1.x + dp1perp * duv2.x;
    vec3 B = dp2perp * duv1.y + dp1perp * duv2.y;

    float invmax = inversesqrt(max(dot(T, T), dot(B, B)));
    return mat3(T * invmax, B * invmax, normal);
}
```

Благодаря этому мы можем вычислить *настоящую* нормаль, другими словами, нормаль поверхности в данном пикселе:

```glsl
mat3 tbn = cotangent_frame(v_normal, v_position, v_tex_coords);
vec3 real_normal = normalize(tbn * -(normal_map * 2.0 - 1.0));
```

Остальная часть кода такая же, как и раньше. Мы применяем затенение по фонгу, за исключением того, что мы используем `real_normal` вместо` v_normal`.

И вот результат:

![Финальный результат](../tuto-14-step3.png)

Это гораздо убедительнее!

**[Вы можете найти весь исходный код здесь](https://github.com/glium/glium/blob/master/examples/tutorial-14.rs).**
