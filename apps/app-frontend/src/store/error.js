import { defineStore } from 'pinia'

export const useError = defineStore('errorsStore', {
	state: () => ({
		errorModal: null,
		pendingError: null,
	}),
	actions: {
		setErrorModal(ref) {
			this.errorModal = ref
			if (ref?.show && this.pendingError) {
				const pendingError = this.pendingError
				this.pendingError = null
				ref.show(
					pendingError.error,
					pendingError.context,
					pendingError.closable,
					pendingError.source,
				)
			}
		},
		showError(error, context, closable = true, source = null) {
			if (this.errorModal?.show) {
				this.errorModal.show(error, context, closable, source)
				return
			}

			this.pendingError = { error, context, closable, source }
		},
	},
})

export const handleSevereError = (err, context) => {
	const error = useError()
	error.showError(err, context)
	console.error(err)
}
