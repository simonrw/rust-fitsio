gentestfiles: testdata/vector_columns.fits

testdata/vector_columns.fits: filegen/vector_columns.py
	python $< -o $@
