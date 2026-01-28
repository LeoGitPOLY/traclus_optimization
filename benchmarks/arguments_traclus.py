DEFAULT_PATHS = [
        "90_degres_3_DL_traclus.txt",
        "montreal_to_montreal_DL_traclus.txt",
        "small_radius_to_small_radius_DL_traclus.txt",
        "up_the_bridges_DL_traclus.txt" ]

DEFAULT_VALUES = {
        'max_dist':     [250],
        'min_density':  [2],
        'max_angle':    [5],
        'seg_size':     [500],
        'path':         DEFAULT_PATHS
    }

class ArgumentsTraclus:
    def __init__(self, data_path: str, values: dict = []):
        self.index_args = 0
        self.index_path = 0
        self.data_path = data_path
        self.values = values
        self.load_arguments()

    def load_arguments(self):
        self._path =         self.__valid_value('path')
        self._max_dist =     self.__valid_value('max_dist')
        self._min_density =  self.__valid_value('min_density')
        self._max_angle =    self.__valid_value('max_angle')
        self._seg_size =     self.__valid_value('seg_size')

        self.max_index_args = max(
            len(self._max_dist),
            len(self._min_density),
            len(self._max_angle),
            len(self._seg_size),
        )

        # 2) Extend all lists to that length
        self._max_dist = self._extend_to_length(self._max_dist, self.max_index_args)
        self._min_density = self._extend_to_length(self._min_density, self.max_index_args)
        self._max_angle = self._extend_to_length(self._max_angle, self.max_index_args)
        self._seg_size = self._extend_to_length(self._seg_size, self.max_index_args)


    def reset_arguments(self):
        self.index_args = 0
        self.index_path = 0
        self.load_arguments()
         
    def iter_arguments(self) -> bool:     
        self.index_args += 1

        if self.index_args >= self.max_index_args:
            self.index_args = 0
            self.index_path += 1

        if self.index_path >= len(self._path):
            self.index_path = -1
            return False

        self.load_arguments()
        return True
    
    def get_path(self) -> str:
        return self.data_path + "/" + self.get_name()

    def get_name(self) -> str:
        return self._path[self.index_path]
    
    def get_args_value(self, name:str) -> str:
        if name == 'max_dist':
            return str(self._max_dist[self.index_args])
        elif name == 'min_density':
            return str(self._min_density[self.index_args])
        elif name == 'max_angle':
            return str(self._max_angle[self.index_args])
        elif name == 'seg_size':
            return str(self._seg_size[self.index_args])
        else:
            raise KeyError(f"Key '{name}' is not a valid argument key.")
    
    def get_args(self) -> str:
        args = (self.get_args_value('max_dist'), self.get_args_value('min_density'), 
                self.get_args_value('max_angle'), self.get_args_value('seg_size'), 
                self.get_name())

        return f"[max dist: {args[0]}, min density: {args[1]}, max angle: {args[2]}, seg size: {args[3]}] for '{args[4]}'"
        
    def __valid_value(self, key: str) -> list:
        if not key in DEFAULT_VALUES:
            raise KeyError(f"Key '{key}' is not a valid argument key.")

        valid_value = True
        
        if not self.values: valid_value = False
        if valid_value and not key in self.values: valid_value = False
        if valid_value and not self.values[key]: valid_value = False

        if valid_value: return self.values[key]
        else: return DEFAULT_VALUES[key]

    def _extend_to_length(self, values: list, target_len: int) -> list:
        if len(values) >= target_len:
            return values
        return values + [values[-1]] * (target_len - len(values))

