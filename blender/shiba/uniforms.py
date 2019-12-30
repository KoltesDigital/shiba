import bpy
from dataclasses import dataclass, field
from shiba import callback_lists
from shiba.library_definitions import Matrix, UniformValue, to_c_matrix
from typing import List


class Uniforms(bpy.types.PropertyGroup):
    pass


@dataclass
class ContextValues:
    inverse_projection: Matrix = None
    inverse_view: Matrix = None
    projection: Matrix = None
    resolution_height: float = None
    resolution_width: float = None
    time: float = None
    view: Matrix = None


@dataclass
class UniformAnnotationDescriptor:
    pass


@dataclass
class UniformAnnotationWithValueDescriptor(UniformAnnotationDescriptor):
    def to_api_uniform_value(self, context_values, uniforms):
        raise NotImplementedError()


@dataclass
class UniformControlAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    name: str = None

    @property
    def property_name(self):
        return 'uniform_%s' % self.name

    def add_property(self):
        pass

    def remove_property(self):
        pass

    def draw_property(self, uniforms, layout):
        pass


@dataclass
class UniformControlCheckboxAnnotationDescriptor(UniformControlAnnotationDescriptor):
    default: bool = False

    def add_property(self):
        setattr(Uniforms, self.property_name, bpy.props.BoolProperty(
            default=self.default,
            name=self.name,
            update=_update,
        ))

    def remove_property(self):
        delattr(Uniforms, self.property_name)

    def draw_property(self, uniforms, layout):
        layout.prop(uniforms, self.property_name)

    def to_api_uniform_value(self, context_values, uniforms):
        value = getattr(uniforms, self.property_name)
        return UniformValue(
            as_float=float(value),
            as_int=int(value),
            as_uint=int(value),
        )


@dataclass
class UniformControlObjectAnnotationDescriptor(UniformControlAnnotationDescriptor):
    def add_property(self):
        setattr(Uniforms, self.property_name, bpy.props.PointerProperty(
            name=self.name,
            type=bpy.types.Object,
            update=_update,
        ))

    def remove_property(self):
        delattr(Uniforms, self.property_name)

    def draw_property(self, uniforms, layout):
        layout.prop(uniforms, self.property_name)

    def to_api_uniform_value(self, context_values, uniforms):
        obj = getattr(uniforms, self.property_name)
        if obj:
            return UniformValue(
                as_mat4=to_c_matrix(obj.matrix_world),
            )
        return UniformValue()


@dataclass
class UniformControlSliderAnnotationDescriptor(UniformControlAnnotationDescriptor):
    default: float = 0.0
    max: float = 3.402823e+38
    min: float = -3.402823e+38

    def add_property(self):
        setattr(Uniforms, self.property_name, bpy.props.FloatProperty(
            default=self.default,
            max=self.max,
            min=self.min,
            name=self.name,
            update=_update,
        ))

    def remove_property(self):
        delattr(Uniforms, self.property_name)

    def draw_property(self, uniforms, layout):
        layout.prop(uniforms, self.property_name)

    def to_api_uniform_value(self, context_values, uniforms):
        value = getattr(uniforms, self.property_name)
        return UniformValue(
            as_float=float(value),
            as_int=int(value),
            as_uint=int(value),
        )


@dataclass
class UniformInverseProjectionAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def to_api_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_mat4=to_c_matrix(context_values.inverse_projection),
        )


@dataclass
class UniformInverseViewAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def to_api_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_mat4=to_c_matrix(context_values.inverse_view),
        )


@dataclass
class UniformProjectionAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def to_api_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_mat4=to_c_matrix(context_values.projection),
        )


@dataclass
class UniformViewAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def to_api_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_mat4=to_c_matrix(context_values.view),
        )


@dataclass
class UniformResolutionHeightAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def to_api_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_float=context_values.resolution_height,
            as_int=int(context_values.resolution_height),
            as_uint=int(context_values.resolution_height),
        )


@dataclass
class UniformResolutionWidthAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def to_api_uniform_value(self, context_values, uniforms):
        return UniformValue(
            as_float=context_values.resolution_width,
            as_int=int(context_values.resolution_width),
            as_uint=int(context_values.resolution_width),
        )


@dataclass
class UniformTimeAnnotationDescriptor(UniformAnnotationWithValueDescriptor):
    def to_api_uniform_value(self, context_values, uniforms):
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


def _parse_control_parameters(str):
    if not str:
        return None

    d = {}
    arr = str.split(',')
    for item in arr:
        pair = item.split('=')
        if len(pair) == 2:
            key = pair[0].strip()
            value = pair[1].strip()
            d[key] = value
    return d


def _make_control_annotation(annotation, name):
    control_parameters = _parse_control_parameters(annotation['control-parameters'])
    if annotation['control-kind'] == 'checkbox':
        args = control_parameters and {
            'default': control_parameters.get('default', 'unchecked') == 'checked'
        } or {}
        return UniformControlCheckboxAnnotationDescriptor(
            name=name,
            **args
        )
    if annotation['control-kind'] == 'object':
        return UniformControlObjectAnnotationDescriptor(
            name=name,
        )
    if annotation['control-kind'] == 'slider':
        args = control_parameters and {
            'default': float(control_parameters.get('default', 0.0)),
            'max': float(control_parameters.get('max', 3.402823e+38)),
            'min': float(control_parameters.get('min', -3.402823e+38)),
        } or {}
        return UniformControlSliderAnnotationDescriptor(
            name=name,
            **args
        )


def _make_inverse_projection_annotation(annotation, name):
    return UniformInverseProjectionAnnotationDescriptor()


def _make_inverse_view_annotation(annotation, name):
    return UniformInverseViewAnnotationDescriptor()


def _make_projection_annotation(annotation, name):
    return UniformProjectionAnnotationDescriptor()


def _make_resolution_height_annotation(annotation, name):
    return UniformResolutionHeightAnnotationDescriptor()


def _make_resolution_width_annotation(annotation, name):
    return UniformResolutionWidthAnnotationDescriptor()


def _make_time_annotation(annotation, name):
    return UniformTimeAnnotationDescriptor()


def _make_view_annotation(annotation, name):
    return UniformViewAnnotationDescriptor()


_make_annotation_handlers = {
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

    def make_annotation(annotation, name):
        handler = _make_annotation_handlers[annotation['kind']]
        return handler(annotation, name)

    for variable in variables:
        if variable['kind'] == 'uniform' and variable['active'] == True:
            name = variable['name']
            uniform_descriptor = UniformDescriptor(
                annotations=[make_annotation(annotation, name) for annotation in variable['annotations']],
                name=name,
                type_name=variable['type-name'],
            )
            _active_uniform_descriptors.append(uniform_descriptor)

        control_annotation = uniform_descriptor.get_annotation(UniformControlAnnotationDescriptor)
        if control_annotation is not None:
            control_annotation.add_property()

    # Redraw the panel, when it'll be available.


def get_api_uniform_values(context_values, uniforms):
    def to_api_uniform_value(uniform_descriptor):
        control_annotation = uniform_descriptor.get_annotation(UniformAnnotationWithValueDescriptor)
        if control_annotation is not None:
            return control_annotation.to_api_uniform_value(context_values, uniforms)
        return UniformValue()

    UniformValueArray = UniformValue * len(_active_uniform_descriptors)
    array = [to_api_uniform_value(uniform_descriptor) for uniform_descriptor in _active_uniform_descriptors]
    uniform_values = UniformValueArray(*array)

    return uniform_values
