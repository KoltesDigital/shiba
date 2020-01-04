import bpy
from dataclasses import dataclass, field
from shiba import callback_lists, library_definitions
from shiba.library_definitions import Mat4, UniformValue
from typing import List


class Uniforms(bpy.types.PropertyGroup):
    pass


@dataclass
class ContextValues:
    inverse_projection: Mat4 = None
    inverse_view: Mat4 = None
    projection: Mat4 = None
    resolution_height: float = None
    resolution_width: float = None
    time: float = None
    view: Mat4 = None


@dataclass
class UniformAnnotationDescriptor:
    pass


@dataclass
class UniformAnnotationWithValueDescriptor(UniformAnnotationDescriptor):
    def get_uniform_value(self, context_values, uniforms):
        raise NotImplementedError()


def _transform_eval(s):
    return eval(s, {}, {})


def _transform_id(s):
    return s


def _make_transform_from_enum(array):
    def transform(value):
        value = value.upper()
        return value if value in array else None
    return transform


_CONTROL_ANNOTATION_SINGLE_SUBTYPES = {'PIXEL', 'UNSIGNED', 'PERCENTAGE', 'FACTOR', 'ANGLE', 'TIME', 'DISTANCE', 'NONE'}
_transform_single_subtype = _make_transform_from_enum(_CONTROL_ANNOTATION_SINGLE_SUBTYPES)

_CONTROL_ANNOTATION_ARRAY_SUBTYPES = {'COLOR', 'TRANSLATION', 'DIRECTION', 'VELOCITY', 'ACCELERATION', 'MATRIX', 'EULER', 'QUATERNION', 'AXISANGLE', 'XYZ', 'COLOR_GAMMA', 'LAYER', 'LAYER_MEMBER', 'POWER', 'NONE'}
_transform_array_subtype = _make_transform_from_enum(_CONTROL_ANNOTATION_ARRAY_SUBTYPES)

_CONTROL_ANNOTATION_UNITS = ['NONE', 'LENGTH', 'AREA', 'VOLUME', 'ROTATION', 'TIME', 'VELOCITY', 'ACCELERATION', 'MASS', 'CAMERA', 'POWER']
_transform_unit = _make_transform_from_enum(_CONTROL_ANNOTATION_UNITS)


@dataclass
class UniformControlAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    name: str = None
    parameters: object = None

    @property
    def property_name(self):
        return self.name

    def add_property(self):
        pass

    def remove_property(self):
        try:
            delattr(Uniforms, self.property_name)
        except AttributeError:
            pass

    def draw_property(self, uniforms, layout):
        layout.prop(uniforms, self.property_name)

    def _make_args(self, **kwargs):
        args = {}
        if self.parameters:
            for key, transform in kwargs.items():
                if key in self.parameters:
                    arg = transform(self.parameters[key])
                    if arg is not None:
                        args[key] = arg
        return args


@dataclass
class UniformControlBoolAnnotationDescriptor(UniformControlAnnotationDescriptor):
    def add_property(self):
        try:
            args = self._make_args(
                default=lambda value: bool(_transform_eval(value)),
                description=_transform_id,
            )
            setattr(Uniforms, self.property_name, bpy.props.BoolProperty(
                name=self.name,
                update=_update,
                **args
            ))
        except:
            pass

    def get_uniform_value(self, context_values, uniforms):
        try:
            value = getattr(uniforms, self.property_name)
            return UniformValue(
                as_int=int(value),
            )
        except:
            return None


@dataclass
class UniformControlFloatAnnotationDescriptor(UniformControlAnnotationDescriptor):
    def add_property(self):
        try:
            args = self._make_args(
                default=_transform_eval,
                description=_transform_id,
                max=_transform_eval,
                min=_transform_eval,
                precision=_transform_eval,
                step=lambda value: _transform_eval(value) * 100,
                subtype=_transform_single_subtype,
                unit=_transform_unit,
            )
            setattr(Uniforms, self.property_name, bpy.props.FloatProperty(
                name=self.name,
                update=_update,
                **args
            ))
        except:
            pass

    def get_uniform_value(self, context_values, uniforms):
        try:
            value = getattr(uniforms, self.property_name)
            return UniformValue(
                as_float=float(value),
            )
        except:
            return None


def _make_UniformControlMatAnnotationDescriptor(size, get_from_obj):
    @dataclass
    class UniformControlMatAnnotationDescriptor(UniformControlAnnotationDescriptor):
        def add_property(self):
            try:
                args = self._make_args(
                    description=_transform_id,
                )
                setattr(Uniforms, self.property_name, bpy.props.PointerProperty(
                    name=self.name,
                    type=bpy.types.Object,
                    update=_update,
                    **args
                ))
            except:
                pass

        def get_uniform_value(self, context_values, uniforms):
            try:
                obj = getattr(uniforms, self.property_name)
                return get_from_obj(obj)
            except:
                return None

    return UniformControlMatAnnotationDescriptor


UniformControlMat2AnnotationDescriptor = _make_UniformControlMatAnnotationDescriptor(2, lambda obj: UniformValue(
    as_mat2=library_definitions.to_mat2(obj.matrix_world) if obj else library_definitions.MAT2_IDENTITY,
))

UniformControlMat3AnnotationDescriptor = _make_UniformControlMatAnnotationDescriptor(3, lambda obj: UniformValue(
    as_mat3=library_definitions.to_mat3(obj.matrix_world) if obj else library_definitions.MAT3_IDENTITY,
))

UniformControlMat4AnnotationDescriptor = _make_UniformControlMatAnnotationDescriptor(4, lambda obj: UniformValue(
    as_mat4=library_definitions.to_mat4(obj.matrix_world) if obj else library_definitions.MAT4_IDENTITY,
))


def _make_UniformControlVecAnnotationDescriptor(size, get_from_value):
    @dataclass
    class UniformControlVecAnnotationDescriptor(UniformControlAnnotationDescriptor):
        def add_property(self):
            try:
                args = self._make_args(
                    default=_transform_eval,
                    description=_transform_id,
                    max=_transform_eval,
                    min=_transform_eval,
                    precision=_transform_eval,
                    step=lambda value: _transform_eval(value) * 100,
                    subtype=_transform_array_subtype,
                    unit=_transform_unit,
                )
                setattr(Uniforms, self.property_name, bpy.props.FloatVectorProperty(
                    name=self.name,
                    size=size,
                    update=_update,
                    **args
                ))
            except:
                pass

        def get_uniform_value(self, context_values, uniforms):
            try:
                value = getattr(uniforms, self.property_name)
                return get_from_value(value)
            except:
                return None

    return UniformControlVecAnnotationDescriptor


UniformControlVec2AnnotationDescriptor = _make_UniformControlVecAnnotationDescriptor(2, lambda value: UniformValue(
    as_vec2=(value[0], value[1]),
))

UniformControlVec3AnnotationDescriptor = _make_UniformControlVecAnnotationDescriptor(3, lambda value: UniformValue(
    as_vec3=(value[0], value[1], value[2]),
))

UniformControlVec4AnnotationDescriptor = _make_UniformControlVecAnnotationDescriptor(4, lambda value: UniformValue(
    as_vec4=(value[0], value[1], value[2], value[3]),
))


@dataclass
class UniformInverseProjectionAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def get_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_mat4=library_definitions.to_mat4(context_values.inverse_projection),
        )


@dataclass
class UniformInverseViewAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def get_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_mat4=library_definitions.to_mat4(context_values.inverse_view),
        )


@dataclass
class UniformProjectionAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def get_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_mat4=library_definitions.to_mat4(context_values.projection),
        )


@dataclass
class UniformViewAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def get_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_mat4=library_definitions.to_mat4(context_values.view),
        )


@dataclass
class UniformResolutionHeightAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def get_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_float=context_values.resolution_height,
            as_int=int(context_values.resolution_height),
            as_uint=int(context_values.resolution_height),
        )


@dataclass
class UniformResolutionWidthAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def get_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_float=context_values.resolution_width,
            as_int=int(context_values.resolution_width),
            as_uint=int(context_values.resolution_width),
        )


@dataclass
class UniformTimeAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def get_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_float=context_values.time,
            as_int=int(context_values.time),
            as_uint=int(context_values.time),
        )


@dataclass
class UniformDescriptor:
    annotations: List[UniformAnnotationDescriptor] = field(default_factory=list)
    name: str = None
    type_name: str = None

    def get_annotation(self, Class):
        for annotation in self.annotations:
            if isinstance(annotation, Class):
                return annotation
        return None


_active_uniform_descriptors = []


def get_active_uniform_descriptors():
    return _active_uniform_descriptors


def _update(_self, _context):
    callback_lists.viewport_update.trigger()


_CONTROL_ANNOTATION_CLASSES = {
    'bool': UniformControlBoolAnnotationDescriptor,
    'float': UniformControlFloatAnnotationDescriptor,
    'mat2': UniformControlMat2AnnotationDescriptor,
    'mat3': UniformControlMat3AnnotationDescriptor,
    'mat4': UniformControlMat4AnnotationDescriptor,
    'vec2': UniformControlVec2AnnotationDescriptor,
    'vec3': UniformControlVec3AnnotationDescriptor,
    'vec4': UniformControlVec4AnnotationDescriptor,
}


def _make_control_annotation(annotation, type_name, name):
    Cls = _CONTROL_ANNOTATION_CLASSES.get(type_name, None)
    if Cls:
        return Cls(
            name=name,
            parameters=annotation['parameters'],
        )
    else:
        print('Unknown control for type %s.' % type_name)


def _make_inverse_projection_annotation(annotation, type_name, name):
    return UniformInverseProjectionAnnotationDescriptor()


def _make_inverse_view_annotation(annotation, type_name, name):
    return UniformInverseViewAnnotationDescriptor()


def _make_projection_annotation(annotation, type_name, name):
    return UniformProjectionAnnotationDescriptor()


def _make_resolution_height_annotation(annotation, type_name, name):
    return UniformResolutionHeightAnnotationDescriptor()


def _make_resolution_width_annotation(annotation, type_name, name):
    return UniformResolutionWidthAnnotationDescriptor()


def _make_time_annotation(annotation, type_name, name):
    return UniformTimeAnnotationDescriptor()


def _make_view_annotation(annotation, type_name, name):
    return UniformViewAnnotationDescriptor()


_MAKE_ANNOTATION_HANDLERS = {
    'control': _make_control_annotation,
    'inverse-projection': _make_inverse_projection_annotation,
    'inverse-view': _make_inverse_view_annotation,
    'projection': _make_projection_annotation,
    'resolution-height': _make_resolution_height_annotation,
    'resolution-width': _make_resolution_width_annotation,
    'time': _make_time_annotation,
    'view': _make_view_annotation,
}


def set_shader_variables(variables):
    _active_uniform_descriptors.clear()

    def make_annotation(annotation, type_name, name):
        handler = _MAKE_ANNOTATION_HANDLERS[annotation['kind']]
        return handler(annotation, type_name, name)

    for variable in variables:
        if variable['kind'] == 'uniform' and variable['active'] == True:
            type_name = variable['type-name']
            name = variable['name']
            uniform_descriptor = UniformDescriptor(
                annotations=[make_annotation(annotation, type_name, name) for annotation in variable['annotations']],
                name=name,
                type_name=type_name,
            )
            _active_uniform_descriptors.append(uniform_descriptor)

        control_annotation = uniform_descriptor.get_annotation(UniformControlAnnotationDescriptor)
        if control_annotation is not None:
            control_annotation.add_property()

    # Redraw the panel, when it'll be available.


def get_uniform_values(context_values, uniforms):
    def get_uniform_value(uniform_descriptor):
        annotation_with_value = uniform_descriptor.get_annotation(UniformAnnotationWithValueDescriptor)
        if annotation_with_value is not None:
            uniform_value = annotation_with_value.get_uniform_value(context_values, uniforms)
            if uniform_value:
                return uniform_value
        return UniformValue()

    UniformValueArray = UniformValue * len(_active_uniform_descriptors)
    array = [get_uniform_value(uniform_descriptor) for uniform_descriptor in _active_uniform_descriptors]
    uniform_values = UniformValueArray(*array)

    return uniform_values
